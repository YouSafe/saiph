use std::{
    marker::PhantomData,
    sync::{
        Arc, Barrier, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{Sender, channel},
    },
};

use crate::{
    ThreadSpawner,
    board::Board,
    clock::Clock,
    evaluation::Evaluation,
    pv::PrincipleVariation,
    search::{NodeCountBuffer, RootMove, Search, ThreadData},
    transposition::TranspositionTable,
    types::search_limits::{SearchLimits, TimeLimit},
    uci::EngineMessage,
};

#[derive(Debug, Default)]
pub struct StopSync {
    /// A flag indicating whether the **current** search should be stopped.
    ///
    /// This flag is reset before each search, meaning setting it before a
    /// search does not affect the subsequent search.
    pub stop: AtomicBool,
    pub wait_for_stop: Mutex<bool>,
    pub cond_var: Condvar,
}

pub struct ThreadPool<S: ThreadSpawner> {
    workers: Vec<WorkerHandle>,
    stop_sync: Arc<StopSync>,
    _marker: PhantomData<S>,
}

impl<S: ThreadSpawner> ThreadPool<S> {
    pub fn new(
        num_threads: u8,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
    ) -> Self {
        let mut workers = Vec::with_capacity(num_threads as usize);

        let stop_sync = Arc::new(StopSync::default());
        let barrier = Arc::new(Barrier::new(num_threads as usize));
        let nodes_buffer = Arc::new(NodeCountBuffer::new(num_threads));

        for id in 0..num_threads {
            workers.push(Self::spawn_worker(
                stop_sync.clone(),
                barrier.clone(),
                num_threads,
                id,
                engine_tx.clone(),
                tt.clone(),
                nodes_buffer.clone(),
            ));
        }

        Self {
            workers,
            stop_sync,
            _marker: Default::default(),
        }
    }

    pub fn search(&self, board: Board, limits: SearchLimits, clock: Clock, multipv: u8) {
        let legal_moves = board.generate_moves();
        let root_moves = if !limits.search_moves.is_empty() {
            limits
                .search_moves
                .iter()
                .filter_map(|sm| legal_moves.iter().find(|m| sm == m))
                .cloned()
                .collect()
        } else {
            legal_moves
        };

        let root_moves: Vec<RootMove> = root_moves
            .into_iter()
            .map(|m| RootMove {
                score: Evaluation::MIN,
                pv: PrincipleVariation::from_root(m),
            })
            .collect();

        // assign workers search job
        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Search(Search::new(
                    board.clone(),
                    limits.clone(),
                    clock,
                    root_moves.clone(),
                    multipv.min(root_moves.len().min(u8::MAX as usize) as u8),
                )))
                .unwrap();
        }
    }

    pub fn stop_search(&self) {
        let mut wait_for_stop = self.stop_sync.wait_for_stop.lock().unwrap();
        *wait_for_stop = false;
        drop(wait_for_stop);
        self.stop_sync.cond_var.notify_all();

        self.stop_sync.stop.store(true, Ordering::SeqCst);
    }

    pub fn resize(
        &mut self,
        num_threads: u8,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
    ) {
        let new_barrier = Arc::new(Barrier::new(num_threads as usize));
        let new_nodes_buffer = Arc::new(NodeCountBuffer::new(num_threads));

        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Resize {
                    new_num_threads: num_threads,
                    new_barrier: new_barrier.clone(),
                    new_nodes_buffer: new_nodes_buffer.clone(),
                })
                .unwrap();
        }

        let mut thread_id = self.workers.len() as u8;
        self.workers.resize_with(num_threads as usize, || {
            let worker = Self::spawn_worker(
                self.stop_sync.clone(),
                new_barrier.clone(),
                num_threads,
                thread_id,
                engine_tx.clone(),
                tt.clone(),
                new_nodes_buffer.clone(),
            );

            thread_id += 1;

            worker
        });
    }

    pub fn update_tt(&self, tt: Arc<TranspositionTable>) {
        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::UpdateTT { tt: tt.clone() })
                .unwrap();
        }
    }

    pub fn reset_data(&self) {
        for worker in &self.workers {
            worker.worker_tx.send(Job::ResetData).unwrap();
        }
    }

    pub fn quit(&self, stop_search: bool) {
        let active_threads = Arc::new(AtomicU8::new(self.workers.len() as u8));

        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Quit {
                    active_threads: active_threads.clone(),
                })
                .unwrap();
        }

        if stop_search {
            self.stop_search();
        }
    }

    pub fn ready(&self) {
        for worker in &self.workers {
            worker.worker_tx.send(Job::Ready {}).unwrap();
        }
    }

    fn spawn_worker(
        stop_sync: Arc<StopSync>,
        barrier: Arc<Barrier>,
        num_threads: u8,
        thread_id: u8,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
        nodes_buffer: Arc<NodeCountBuffer>,
    ) -> WorkerHandle {
        let (worker_tx, worker_rx) = channel();
        S::spawn(move || {
            let thread_data = ThreadData {
                engine_tx,
                tt,
                stop_sync: stop_sync.clone(),
                nodes_buffer,
                thread_id,
            };

            worker_loop(
                stop_sync,
                thread_id,
                worker_rx,
                barrier,
                num_threads,
                thread_data,
            );
        });

        WorkerHandle { worker_tx }
    }
}

fn worker_loop(
    stop_sync: Arc<StopSync>,
    thread_id: u8,
    worker_rx: std::sync::mpsc::Receiver<Job>,
    mut barrier: Arc<Barrier>,
    mut num_threads: u8,
    mut thread_data: ThreadData,
) {
    while let Ok(job) = worker_rx.recv() {
        // Wait for previous job to finish on all workers
        let wait = barrier.wait();
        match job {
            Job::Search(search) => {
                if wait.is_leader() {
                    thread_data.nodes_buffer.clear();

                    stop_sync.stop.store(false, Ordering::SeqCst);

                    let mut wait_for_stop = stop_sync.wait_for_stop.lock().unwrap();
                    // set to false if not infinite search or ponder
                    *wait_for_stop = search.limits.time == TimeLimit::Infinite;
                    drop(wait_for_stop);
                }

                let wait = barrier.wait();
                search.run(&mut thread_data, wait.is_leader());
            }
            Job::Resize {
                new_num_threads,
                new_barrier,
                new_nodes_buffer,
            } => {
                num_threads = new_num_threads;
                barrier = new_barrier;
                thread_data.nodes_buffer = new_nodes_buffer;
            }
            Job::ResetData => {
                // SAFETY: synchronisation and unique threads ensure that each thread
                // has exclusive access on their respective chunk
                unsafe {
                    thread_data
                        .tt
                        .clear_chunk(thread_id as usize, num_threads as usize)
                };
            }
            Job::UpdateTT { tt } => {
                thread_data.tt = tt;

                // SAFETY: synchronisation and unique threads ensure that each thread
                // has exclusive access on their respective chunk
                unsafe {
                    thread_data
                        .tt
                        .clear_chunk(thread_id as usize, num_threads as usize)
                };
            }
            Job::Quit { active_threads } => {
                let previous_value = active_threads.fetch_sub(1, Ordering::SeqCst);
                if previous_value == 1 {
                    thread_data
                        .engine_tx
                        .send(EngineMessage::Terminate)
                        .unwrap();
                }
                break;
            }
            Job::Ready => {
                if wait.is_leader() {
                    thread_data.engine_tx.send(EngineMessage::Ready).unwrap();
                }
            }
        }
    }
}

enum Job {
    Search(Search),
    Resize {
        new_num_threads: u8,
        new_barrier: Arc<Barrier>,
        new_nodes_buffer: Arc<NodeCountBuffer>,
    },
    Quit {
        active_threads: Arc<AtomicU8>,
    },
    ResetData,
    Ready,
    UpdateTT {
        tt: Arc<TranspositionTable>,
    },
}

struct WorkerHandle {
    worker_tx: Sender<Job>,
}
