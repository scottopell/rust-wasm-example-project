extern crate wasmtime;
use std::error::Error;
use wasmtime::*;

const WASM_BYTES_PATH: &str = "/Users/scott.opell/dev/rust-wasm-example-proj/adding-machine/target/wasm32-unknown-unknown/debug/addingmachine.wasm";

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, WASM_BYTES_PATH)?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;

    let add = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "add_wasm")
        .expect("`add_wasm` was not an exported function");

    // And finally we can call our function! Note that the error propagation
    // with `?` is done to handle the case where the wasm function traps.
    let result = add.call(&mut store, (3, 8))?;
    println!("Answer: {:?}", result);
    Ok(())
}
