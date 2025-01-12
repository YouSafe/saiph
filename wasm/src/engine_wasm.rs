use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};

use engine::{board::Board, Printer, SearchWorkerPool};
use engine::{
    uci::EngineUCI, search::Search, search_limits::SearchLimits,
    transposition_table::TranspositionTable,
};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, DedicatedWorkerGlobalScope, MessageChannel, MessagePort, WorkerOptions};

/// Printer for the parent webworker
struct EnginePrinter {
    worker_scope: DedicatedWorkerGlobalScope,
}

impl EnginePrinter {
    pub fn new(worker_scope: DedicatedWorkerGlobalScope) -> Self {
        Self { worker_scope }
    }
}

impl Printer for EnginePrinter {
    fn print(&self, s: &str) {
        self.worker_scope
            .post_message(&JsValue::from_str(s))
            .unwrap();
    }
}

/// Printer for the search worker webworker communicating with the parent webworker
struct SearchPrinter {
    port: MessagePort,
}

impl SearchPrinter {
    pub fn new(port: MessagePort) -> Self {
        Self { port }
    }
}

impl Printer for SearchPrinter {
    fn print(&self, s: &str) {
        self.port.post_message(&JsValue::from_str(s)).unwrap();
    }
}

#[wasm_bindgen]
pub struct Engine {
    engine_uci: EngineUCI<WasmSearchWorkerPool, EnginePrinter>,
    _message_channel: MessageChannel,
}

enum SearchWorkerPoolMessage {
    NewSearchTask(Board, SearchLimits),
    Quit,
}

pub struct WasmSearchWorkerPool {
    channel_sender: Sender<SearchWorkerPoolMessage>,
    table: Arc<Mutex<TranspositionTable>>,
    stop: Arc<AtomicBool>,
    main_thread_handle: Option<web_sys::Worker>,
}

pub fn spawn(
    f: impl FnOnce(MessagePort),
    message_port: MessagePort,
) -> Result<web_sys::Worker, JsValue> {
    let options = WorkerOptions::new();
    options.set_type(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options("./searcher-worker.js", &options)?;
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce(MessagePort)>));

    let transfer_ownership = js_sys::Array::new();

    let msg = js_sys::Array::new();
    msg.push(&wasm_bindgen::module());
    msg.push(&wasm_bindgen::memory());
    msg.push(&JsValue::from(ptr as u32));
    msg.push(&message_port);
    transfer_ownership.push(&message_port);

    worker
        .post_message_with_transfer(&msg, &transfer_ownership)
        .unwrap();

    Ok(worker)
}

#[wasm_bindgen]
pub fn worker_entry_point(addr: u32, message_port: MessagePort) {
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce(MessagePort)>) };
    (*closure)(message_port);
}

impl WasmSearchWorkerPool {
    fn new(message_port: MessagePort) -> Self {
        let (sender, receiver) = mpsc::channel();

        let table = Arc::new(Mutex::new(TranspositionTable::new()));
        let stop = Arc::new(AtomicBool::new(false));

        Self {
            channel_sender: sender,
            table: table.clone(),
            stop: stop.clone(),
            main_thread_handle: Some(
                spawn(
                    move |p_message_port| {
                        let search_printer = SearchPrinter::new(p_message_port);

                        loop {
                            let message = receiver.recv().unwrap();

                            match message {
                                SearchWorkerPoolMessage::Quit => {
                                    eprintln!("not accepting any more search requests");
                                    break;
                                }
                                SearchWorkerPoolMessage::NewSearchTask(board, limits) => {
                                    stop.store(false, Ordering::SeqCst);
                                    let stop_ref = stop.as_ref();
                                    let table_ref = &mut table.lock().unwrap();

                                    let search = Search::new(
                                        board,
                                        table_ref,
                                        stop_ref,
                                        &search_printer,
                                        limits,
                                    );

                                    let pick = search.find_best_move();
                                    if let Some(bestmove) = pick {
                                        search_printer
                                            .print(format!("bestmove {}", bestmove).as_str());
                                    }
                                }
                            }
                        }
                    },
                    message_port,
                )
                .unwrap(),
            ),
        }
    }
}

impl SearchWorkerPool for WasmSearchWorkerPool {
    fn clear_tables(&mut self) {
        self.table.lock().unwrap().clear();
    }

    fn initiate_search(&self, board: Board, limits: SearchLimits) {
        self.channel_sender
            .send(SearchWorkerPoolMessage::NewSearchTask(board, limits))
            .expect("could not send new search task");
    }

    fn stop_search(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Drop for WasmSearchWorkerPool {
    fn drop(&mut self) {
        eprintln!("shutting down searcher thread");
        self.stop_search();
        self.channel_sender
            .send(SearchWorkerPoolMessage::Quit)
            .expect("could not send quit message");
        if let Some(handle) = self.main_thread_handle.take() {
            handle.terminate()
        }
    }
}

#[wasm_bindgen]
impl Engine {
    pub fn new() -> Engine {
        let engine_printer =
            EnginePrinter::new(js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>());

        let message_channel = MessageChannel::new().unwrap();

        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MessageEvent| {
            js_sys::global()
                .unchecked_into::<DedicatedWorkerGlobalScope>()
                .post_message(&event.data())
                .unwrap();
        });

        message_channel
            .port1()
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));

        closure.forget();

        let wasm_searcher = WasmSearchWorkerPool::new(message_channel.port2());

        Engine {
            engine_uci: EngineUCI::new(wasm_searcher, engine_printer),
            _message_channel: message_channel,
        }
    }

    pub fn receive_command(&mut self, message: &str) {
        self.engine_uci.receive_command(message);
    }
}
