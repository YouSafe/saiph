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
    search::{NodeCountBuffer, Search},
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
    workers: Vec<Worker>,
    stop_sync: Arc<StopSync>,
    _marker: PhantomData<S>,
}

impl<S: ThreadSpawner> ThreadPool<S> {
    pub fn new(num_threads: u8) -> Self {
        let mut workers = Vec::with_capacity(num_threads as usize);

        let stop_sync = Arc::new(StopSync::default());
        let barrier = Arc::new(Barrier::new(num_threads as usize));

        for id in 0..num_threads {
            workers.push(Worker::new::<S>(
                stop_sync.clone(),
                barrier.clone(),
                num_threads,
                id,
            ));
        }

        Self {
            workers,
            stop_sync,
            _marker: Default::default(),
        }
    }

    pub fn search(
        &self,
        board: Board,
        limits: SearchLimits,
        clock: Clock,
        engine_tx: Sender<EngineMessage>,
        tt: Arc<TranspositionTable>,
    ) {
        fn send_response(engine_tx: &Sender<EngineMessage>, message: &str) {
            engine_tx
                .send(EngineMessage::Response(message.to_owned()))
                .unwrap();
        }

        let legal_moves = board.generate_moves();
        if legal_moves.is_empty() {
            if !board.checkers().is_empty() {
                send_response(&engine_tx, "info depth 0 score mate 0");
            } else {
                send_response(&engine_tx, "info depth 0 score cp 0");
            }
            send_response(&engine_tx, "bestmove (none)");
            return;
        }

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

        if root_moves.is_empty() {
            send_response(&engine_tx, "info depth 0");
            send_response(&engine_tx, "bestmove (none)");
            return;
        }

        let nodes_buffer = Arc::new(NodeCountBuffer::new(self.workers.len() as u8));

        // assign workers search job
        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Search(Search::new(
                    board.clone(),
                    limits.clone(),
                    clock,
                    root_moves.clone(),
                    engine_tx.clone(),
                    tt.clone(),
                    self.stop_sync.clone(),
                    worker.thread_id,
                    nodes_buffer.clone(),
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

    pub fn resize(&mut self, num_threads: u8) {
        let new_barrier = Arc::new(Barrier::new(num_threads as usize));

        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Resize {
                    new_num_threads: num_threads,
                    new_barrier: new_barrier.clone(),
                })
                .unwrap();
        }

        let mut thread_id = self.workers.len() as u8;
        self.workers.resize_with(num_threads as usize, || {
            let worker = Worker::new::<S>(
                self.stop_sync.clone(),
                new_barrier.clone(),
                num_threads,
                thread_id,
            );

            thread_id += 1;

            worker
        });
    }

    pub fn quit(&self, engine_tx: Sender<EngineMessage>, stop_search: bool) {
        let active_threads = Arc::new(AtomicU8::new(self.workers.len() as u8));

        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Quit {
                    active_threads: active_threads.clone(),
                    engine_tx: engine_tx.clone(),
                })
                .unwrap();
        }

        if stop_search {
            self.stop_search();
        }
    }
}

enum Job {
    Search(Search),
    Resize {
        new_num_threads: u8,
        new_barrier: Arc<Barrier>,
    },
    Quit {
        active_threads: Arc<AtomicU8>,
        engine_tx: Sender<EngineMessage>,
    },
}

struct Worker {
    worker_tx: Sender<Job>,
    thread_id: u8,
}

impl Worker {
    pub fn new<S: ThreadSpawner>(
        stop_sync: Arc<StopSync>,
        barrier: Arc<Barrier>,
        num_threads: u8,
        thread_id: u8,
    ) -> Self {
        let (worker_tx, worker_rx) = channel();

        S::spawn(move || {
            let mut barrier = barrier;
            let mut _num_threads = num_threads;

            while let Ok(job) = worker_rx.recv() {
                match job {
                    Job::Search(search) => {
                        let wait = barrier.wait();
                        if wait.is_leader() {
                            stop_sync.stop.store(false, Ordering::SeqCst);

                            let mut wait_for_stop = stop_sync.wait_for_stop.lock().unwrap();
                            // set to false if not infinite search or ponder
                            *wait_for_stop = search.limits().time == TimeLimit::Infinite;
                            drop(wait_for_stop);
                        }

                        let wait = barrier.wait();
                        search.search(wait.is_leader());

                        barrier.wait();
                    }
                    Job::Resize {
                        new_num_threads,
                        new_barrier,
                    } => {
                        _num_threads = new_num_threads;
                        barrier = new_barrier
                    }
                    Job::Quit {
                        active_threads,
                        engine_tx,
                    } => {
                        let previous_value = active_threads.fetch_sub(1, Ordering::SeqCst);
                        if previous_value == 1 {
                            engine_tx.send(EngineMessage::Terminate).unwrap();
                        }
                        break;
                    }
                }
            }
        });

        Self {
            worker_tx,
            thread_id,
        }
    }
}
