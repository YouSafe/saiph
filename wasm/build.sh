RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  rustup run nightly-x86_64-unknown-linux-gnu \
  wasm-pack build --target web --out-dir ../pkg . \
  -- -Z build-std=panic_abort,std