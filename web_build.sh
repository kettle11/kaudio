RUSTFLAGS='-C target-feature=+atomics,+bulk-memory' \
  cargo build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort --example file
cp target/wasm32-unknown-unknown/release/examples/file.wasm examples/web_build/sine.wasm