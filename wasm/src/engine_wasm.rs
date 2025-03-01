use std::sync::mpsc::{channel, Sender};

use engine::uci::EngineUCI;
use engine::{uci::EngineMessage, Printer, ThreadSpawner};
use wasm_bindgen::prelude::*;
use web_sys::{
    js_sys::{self, Function},
    Blob, DedicatedWorkerGlobalScope, MessageEvent, Url, Worker, WorkerOptions,
};

struct WebWorkerSpawner;
impl ThreadSpawner for WebWorkerSpawner {
    fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        spawn_worker(f).unwrap();
    }
}

struct PostMessagePrinter;
impl Printer for PostMessagePrinter {
    fn println(s: &str) {
        js_sys::global()
            .unchecked_into::<DedicatedWorkerGlobalScope>()
            .post_message(&JsValue::from_str(s))
            .unwrap();
    }
}

#[wasm_bindgen]
pub struct Engine {
    engine_tx: Sender<EngineMessage>,
    worker: Worker,
    closure: Option<Closure<dyn Fn(MessageEvent)>>,
}

pub fn spawn_worker<F>(f: F) -> Result<web_sys::Worker, JsValue>
where
    F: FnOnce() + Send + 'static,
{
    let options = WorkerOptions::new();
    options.set_type(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options(worker_blob_url().as_str(), &options)?;
    let ptr = Box::into_raw(Box::new(Box::new(f) as Box<dyn FnOnce()>));

    let msg = js_sys::Array::new();
    msg.push(&wasm_bindgen::module());
    msg.push(&wasm_bindgen::memory());
    msg.push(&JsValue::from(ptr as u32));

    worker.post_message(&msg).unwrap();

    Ok(worker)
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

#[allow(clippy::new_without_default)]
#[wasm_bindgen]
impl Engine {
    pub fn new() -> Engine {
        let (engine_tx, engine_rx) = channel();

        let worker = spawn_worker({
            let engine_tx = engine_tx.clone();
            move || {
                let engine = EngineUCI::<WebWorkerSpawner, PostMessagePrinter>::new(engine_tx);
                engine.run(engine_rx);
            }
        })
        // FIXME: fix error handling
        .unwrap();

        Engine {
            engine_tx,
            worker,
            closure: None,
        }
    }

    pub fn send_command(&self, message: String) {
        self.engine_tx
            .send(EngineMessage::Command(message))
            .unwrap()
    }

    pub fn set_callback(&mut self, function: Function) {
        let closure = Closure::<dyn Fn(_)>::new(move |event: web_sys::MessageEvent| {
            function.call1(&JsValue::null(), &event.data()).unwrap();
        });

        self.worker
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));

        self.closure.replace(closure);
    }
}
