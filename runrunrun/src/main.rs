// this is the host rust program
extern crate wasmtime;
use std::{error::Error};
use wasmtime::*;

// maybe not needed but will be the the formatted output glue
use serde::{Deserialize, Serialize};


// use std::slice;

const WASM_BYTES_PATH: &str = "/Users/jon.padilla/Documents/scott-rust-wasmtime/rust-wasm-example-project/adding-machine/target/wasm32-unknown-unknown/debug/addingmachine.wasm";
const BUF_SIZE: u32 = 2048;
// const WASM_PAGE_SIZE: u32 = 65536;

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, WASM_BYTES_PATH)?;

    // print out the imports here
    let imports = module.imports();
    println!("printing out all the required imports now:");
    imports.for_each(|import| println!("{0}", import.name()));


    // snippet to print out the exports here
    // let _exports = module.exports();
    // println!("printing out all the exports now: ");
    // exports.for_each(|export| println!("{0}", export.name()));

    // we cannot instantiate like normal since we are missing
    // the below imports
    // __wbindgen_describe
    // __wbindgen_externref_table_grow
    // __wbindgen_externref_table_set_null
    // __wbindgen_throw

    // we can still make the wasm binary run by calling the
    // define_unknown_imports_as_traps() function, but it 
    // requires a linker object first which we create below
    let mut linker = Linker::new(&engine);
    linker.define_unknown_imports_as_traps(&module)?;


    let mut store = Store::new(&engine, ());

    // because we had to use a linker, we instantiate from the linker
    // object instead of Instance::new()

    //let instance = Instance::new(&mut store, &module, &[])?;
    let instance = linker.instantiate(&mut store, &module)?;

    // Load up our exports from the wasmtime instance
    let memory = instance.
        get_export(&mut store, "memory").unwrap()
        .into_memory().unwrap();

    let _memory_buf = memory.data(&mut store);

    let allocate = instance
        .get_typed_func::<u32, u32>(&mut store, "allocate")
        .expect("`allocate` was not an exported function");

    // we theoretically would call deallocate but this example is incomplete
    let _deallocate = instance
        .get_typed_func::<(u32, u32), ()>(&mut store, "deallocate")
        .expect("`deallocate` was not an exported function");

    let allocate_leaked_ptr = allocate.call(&mut store, BUF_SIZE)?;
    let outgoing_str = String::from("Value created by runrunrun/src/main.rs");
    let _res = memory.write(&mut store, allocate_leaked_ptr as usize, outgoing_str.as_bytes());

    // call read_string in wasm module
    let read_string = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "read_string")
        .expect("`read_string was not an exported function`");
    
    let ans = read_string.call(&mut store, (allocate_leaked_ptr, outgoing_str.len() as u32))?;
    println!("wasm output: {ans}");

    unsafe{
        let mut temp_vec = vec![0u8; BUF_SIZE as usize];
        let _slice = memory.read(&mut store, allocate_leaked_ptr as usize, &mut temp_vec).unwrap();
        let utf8 = std::str::from_utf8_unchecked(temp_vec.as_slice());
        println!("wasm output string: {0}", utf8);
    }


    // now trying to run wasm 
    // we will pass in input like like
    /*

    json: 
    {
        program: "string here that is a vrl program"
        event: {another json thingy here that will be modified}
    }

    we need to pass it in as a string

     */

    let run_vrl_wasm = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "run_vrl_wasm")
        .expect("`run_vrl_wasm was not an exported function`");
    
    let vrl_input = serde_json::json!({
        "program": r#".jon = "padilla""#,
        "event": {}
    });

    let vrl_input_str = vrl_input.to_string();

    memory.write(&mut store, allocate_leaked_ptr as usize, vrl_input_str.as_bytes()).unwrap();

    let output_len = run_vrl_wasm.call(&mut store, (allocate_leaked_ptr, vrl_input_str.len() as u32));
    unsafe{
        let mut temp_vec = vec![0u8; BUF_SIZE as usize];
        let _slice = memory.read(&mut store, allocate_leaked_ptr as usize, &mut temp_vec).unwrap();
        let utf8 = std::str::from_utf8_unchecked(temp_vec.as_slice());
        println!("wasm output string: {0}", utf8);
    }
    Ok(())
}
