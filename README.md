# Running rust via wasm in rust

`adding-machine` is a library implemented in Rust that adds two numbers together.

`runrunrun` is a binary implemented in Rust that uses `wasmtime` to run `adding-machine`.

> Note that the path to the wasm bytes is hard-coded in `runrunrun/main.rs`.
> Update this to reflect wherever you checked this code out.

## Building
1) `cd adding-machine && cargo build --target wasm32-unknown-unknown`
2) `cd runrunrun && cargo build`

## Running
`cd runrunrun && cargo run`
