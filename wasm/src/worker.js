import init, { worker_entry_point } from 'WASM_BINDGEN_SHIM_URL'

self.onmessage = async (event) => {
  const [module, memory, ptr] = event.data

  let instance
  try {
    instance = await init(module, memory)
  } catch (err) {
    console.log(err)
  }

  worker_entry_point(Number(ptr))

  // See https://github.com/tweag/rust-wasm-threads/blob/9a2c8430ccec1f78b88e4bf7ec0c6f4ccf2791e5/shared-memory/worker.js#L28-L37
  if (instance) {
    instance.__wbindgen_thread_destroy()
  }

  close()
}
