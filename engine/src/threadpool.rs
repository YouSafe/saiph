use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Barrier, Condvar, Mutex,
    },
};

use crate::{
    board::Board,
    clock::Clock,
    search::{NodeCountBuffer, Search},
    transposition::TranspositionTable,
    types::search_limits::{SearchLimits, TimeLimit},
    uci::EngineMessage,
    ThreadSpawner,
};

#[derive(Debug, Default)]
pub struct StopSync {
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
        // age tt
        // generate rootmoves

        // if #rootmoves = 0 skip search

        let nodes_buffer = Arc::new(NodeCountBuffer::new(self.workers.len() as u8));

        // assign workers search job
        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Search(Search::new(
                    board.clone(),
                    limits.clone(),
                    clock,
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

    pub fn clear(&self, tt: Arc<TranspositionTable>) {
        for worker in &self.workers {
            worker.worker_tx.send(Job::Clear(tt.clone())).unwrap();
        }
    }

    pub fn quit(&self, engine_tx: Sender<EngineMessage>) {
        let signal_count = Arc::new((Mutex::new(0u8), Condvar::new()));

        for worker in &self.workers {
            worker
                .worker_tx
                .send(Job::Quit(signal_count.clone()))
                .unwrap();
        }

        self.stop_search();

        let _guard = signal_count
            .1
            .wait_while(signal_count.0.lock().unwrap(), |signal_count| {
                *signal_count != self.workers.len() as u8
            });

        engine_tx.send(EngineMessage::Terminate).unwrap();
    }
}

enum Job {
    Search(Search),
    Resize {
        new_num_threads: u8,
        new_barrier: Arc<Barrier>,
    },
    Clear(Arc<TranspositionTable>),
    Quit(Arc<(Mutex<u8>, Condvar)>),
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
            let mut num_threads = num_threads;

            loop {
                println!("[{thread_id}] waiting for job!");
                match worker_rx.recv() {
                    Ok(job) => match job {
                        Job::Search(search) => {
                            let wait = barrier.wait();
                            if wait.is_leader() {
                                stop_sync.stop.store(false, Ordering::SeqCst);

                                let mut wait_for_stop = stop_sync.wait_for_stop.lock().unwrap();
                                // set to false if not infinite search or ponder
                                // FIXME: time limit is also infinite if max depth, etc. is specified
                                // we don't want to wait for stop in this case
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
                            num_threads = new_num_threads;
                            barrier = new_barrier
                        }
                        Job::Clear(transposition_table) => {
                            barrier.wait();

                            let chunk =
                                transposition_table.chunk(thread_id as usize, num_threads as usize);

                            for val in chunk {
                                val.store(0, Ordering::Relaxed);
                            }

                            barrier.wait();
                        }
                        Job::Quit(signal_count_pair) => {
                            let mut signal_count = signal_count_pair.0.lock().unwrap();
                            *signal_count += 1;
                            drop(signal_count);

                            signal_count_pair.1.notify_one();
                            break;
                        }
                    },
                    Err(_) => {
                        break;
                    }
                }

                println!("[{thread_id}] finished job!");
            }

            println!("[{thread_id}] terminating worker!");
        });

        Self {
            worker_tx,
            thread_id,
        }
    }
}
