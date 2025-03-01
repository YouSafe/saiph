/// Idea from: https://github.com/chemicstry/wasm_thread/blob/7ec48686bb1a0d9bd42cf16e46622746e4d12ab3/src/wasm32/js/script_path.js

/// Extracts current script file path from artificially generated stack trace
function script_path() {
    try {
      throw new Error()
    } catch (e) {
      let parts = e.stack.match(/(?:\(|@)(\S+):\d+:\d+/)
      return parts[1]
    }
  }
  
  script_path()
  