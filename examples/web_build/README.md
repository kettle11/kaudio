To build this example install `wasmbindgen-cli`

```bash
cargo build --example hello --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/examples/hello.wasm --out-dir examples/web_build --out-name example --no-modules
```

If you do not already have `wasm-bindgen` installed you can install it with:

```bash
cargo install wasm-bindgen-cli
```

Then use a local development server to host the contents of the `examples/web_build/` folder.

You can use `devserver` which can be installed with `cargo install devserver` and run by running `devserver` within this folder.

For Audio Worklets two header flags need to be enabled:

`devserver --header Cross-Origin-Opener-Policy='same-origin' --header Cross-Origin-Embedder-Policy='require-corp'`