use std::sync::mpsc::{Sender, channel, sync_channel};

use engine::uci::EngineUCI;
use engine::{Printer, ThreadSpawner, uci::EngineMessage};
use wasm_bindgen::prelude::*;
use web_sys::Window;
use web_sys::{
    Blob, DedicatedWorkerGlobalScope, MessageEvent, Url, Worker, WorkerOptions,
    js_sys::{self},
};

#[wasm_bindgen]
pub struct Engine {
    engine_tx: Sender<EngineMessage>,
    main_worker: MainWorker,
    listener_closure: Option<Closure<dyn Fn(MessageEvent)>>,
}

// https://github.com/wasm-bindgen/wasm-bindgen/issues/3628#issuecomment-1729033076
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "(message: string) => void")]
    pub type ListenerFunc;
}

#[allow(clippy::new_without_default)]
#[wasm_bindgen]
impl Engine {
    pub fn new() -> Engine {
        let (engine_tx, engine_rx) = channel();

        let main_worker = spawn_main_worker({
            type WasmUCIEngine = EngineUCI<MainThreadSpawnDelegator, PostMessagePrinter>;

            /// Print messages to UI Thread via the postMessage API.
            ///
            /// The postMessage API is used to keep the UI Thread unblocked.
            ///
            /// It is assumed that the UI Thread has set up an Event Listener that listens
            /// to messages sent from the Main Worker. This listener should be registered 
            /// using `Engine::set_callback`.
            ///
            /// This function may only be called from the Main Worker.
            struct PostMessagePrinter;
            impl Printer for PostMessagePrinter {
                fn println(s: &str) {
                    js_sys::global()
                        .unchecked_into::<DedicatedWorkerGlobalScope>()
                        .post_message(&JsValue::from_str(s))
                        .unwrap();
                }
            }

            /// Delegate spawning of Workers to the UI Thread
            ///
            /// We delegate the spawning of Worker to the UI Thread to avoid the
            /// problem where we try to create a new Worker on a blocked Worker
            /// resulting in a deadlock.
            ///
            /// This happens because the creation of Worker in JavaScript is
            /// done asynchronously and relies on the Threads Event Loop to execute tasks.
            ///
            /// Since we never block the UI Thread, it is safe to create the Worker there.
            ///
            /// To make the spawning of Worker synchronous, we utilize a zero-sized
            /// sync channel to ensure that we block until the new Worker is created.
            ///
            /// This function may only be called on the Main Worker Thread, which listens
            /// for requests to create new Workers.
            struct MainThreadSpawnDelegator;
            impl ThreadSpawner for MainThreadSpawnDelegator {
                fn spawn<F>(f: F)
                where
                    F: FnOnce() + Send + 'static,
                {
                    let (tx, rx) = sync_channel(0);

                    let f = move || {
                        tx.send(()).unwrap();
                        f();
                    };

                    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));

                    // send message to UI Thread
                    let msg = js_sys::Array::new();
                    msg.push(&wasm_bindgen::module());
                    msg.push(&wasm_bindgen::memory());
                    msg.push(&JsValue::from(ptr as u32));

                    js_sys::global()
                        .unchecked_into::<DedicatedWorkerGlobalScope>()
                        .post_message(&msg)
                        .unwrap();

                    // wait for Thread B to start
                    rx.recv().unwrap();
                }
            }

            let engine_tx = engine_tx.clone();

            move || {
                let engine = WasmUCIEngine::new(engine_tx);
                engine.run(engine_rx);
            }
        })
        .unwrap();

        Engine {
            engine_tx,
            main_worker,
            listener_closure: None,
        }
    }

    pub fn send_command(&self, message: String) {
        self.engine_tx
            .send(EngineMessage::Command(message))
            .unwrap()
    }

    pub fn set_callback(&mut self, listener: ListenerFunc) {
        let closure = Closure::<dyn Fn(_)>::new(move |event: web_sys::MessageEvent| {
            // We detect messages from the Main Worker to the UI Thread that
            // request the creation of new workers by checking if the data is an array.
            if event.data().is_array() {
                return;
            }
            listener.call1(&JsValue::null(), &event.data()).unwrap();
        });

        // listens to messages from the worker
        self.main_worker
            .inner
            .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
            .unwrap();

        if let Some(old_closure) = self.listener_closure.replace(closure) {
            self.main_worker
                .inner
                .remove_event_listener_with_callback(
                    "message",
                    old_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    }
}

struct MainWorker {
    inner: Worker,
    blob_url: String,
    _delegation_closure: Closure<dyn Fn(MessageEvent) + 'static>,
}

impl Drop for MainWorker {
    fn drop(&mut self) {
        Url::revoke_object_url(&self.blob_url).unwrap();
    }
}

/// Spawn Main Worker
///
/// Spawns a new Worker and registers an Event Listener to receive
/// request for Worker creation.
///  
/// This function may only be called on the UI Thread.
fn spawn_main_worker<F>(f: F) -> Result<MainWorker, JsValue>
where
    F: FnOnce() + Send + 'static,
{
    // called on the UI Thread
    assert!(js_sys::global().is_instance_of::<Window>());

    let options = WorkerOptions::new();
    options.set_type(web_sys::WorkerType::Module);
    let blob_url = worker_blob_url();
    let worker = web_sys::Worker::new_with_options(&blob_url, &options)?;

    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));

    let msg = js_sys::Array::new();
    msg.push(&wasm_bindgen::module());
    msg.push(&wasm_bindgen::memory());
    msg.push(&JsValue::from(ptr as u32));

    let blob_url_clone = blob_url.clone();
    let closure = Closure::<dyn Fn(_)>::new(move |event: web_sys::MessageEvent| {
        // We detect messages from the Main Worker to the UI Thread that
        // request the creation of new workers by checking if the data is an array.
        if !event.data().is_array() {
            return;
        }
        let new_worker = web_sys::Worker::new_with_options(&blob_url_clone, &options).unwrap();

        new_worker.post_message(&event.data()).unwrap();
    });

    // listens to messages from the worker
    worker
        .add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())
        .unwrap();

    worker.post_message(&msg).unwrap();

    Ok(MainWorker {
        inner: worker,
        _delegation_closure: closure,
        blob_url,
    })
}

#[wasm_bindgen]
pub fn worker_entry_point(addr: u32) {
    let closure = unsafe { Box::from_raw(addr as *mut Box<dyn FnOnce()>) };
    (*closure)();
}

pub fn get_wasm_bindgen_shim_script_path() -> String {
    js_sys::eval(include_str!("script_path.js"))
        .unwrap()
        .as_string()
        .unwrap()
}

pub fn worker_blob_url() -> String {
    let wasm_bindgen_shim_url = get_wasm_bindgen_shim_script_path();

    let template = include_str!("worker.js");
    let script = template.replace("WASM_BINDGEN_SHIM_URL", &wasm_bindgen_shim_url);

    let arr = js_sys::Array::new();
    arr.set(0, JsValue::from_str(&script));
    let blob = Blob::new_with_str_sequence(&arr).unwrap();
    Url::create_object_url_with_blob(
        &blob
            .slice_with_f64_and_f64_and_content_type(0.0, blob.size(), "text/javascript")
            .unwrap(),
    )
    .unwrap()
}
