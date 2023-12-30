pub mod engine_wasm;

extern crate console_error_panic_hook;
use std::panic;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/snippet.js")]
extern "C" {
    fn output(s: &str);
}

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}
