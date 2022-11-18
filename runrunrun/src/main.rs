// this is the host rust program
extern crate wasmtime;
use std::{error::Error, ptr::{copy, write_bytes}};
use wasmtime::*;
use std::slice;

const WASM_BYTES_PATH: &str = "/Users/jon.padilla/Documents/scott-rust-wasmtime/rust-wasm-example-project/adding-machine/target/wasm32-unknown-unknown/debug/addingmachine.wasm";
const BUF_SIZE: u32 = 2048;
const WASM_PAGE_SIZE: u32 = 65536;


unsafe fn writeToWasmMemory(outgoing: String, bufPtr: u32, memory: Memory, store: &mut Store<()>, instance_ref: &mut Instance) -> Result<(), Box<dyn Error>> {
    if outgoing.len() as u32 > BUF_SIZE {
        println!("runrunrun/src/main.rs::writeToWasmMemory() : outgoing string length {0} bigger than buffer {1}", outgoing.len(), BUF_SIZE);
    }
    
    // let memory = instance_ref.get_export(store, "memory").unwrap().into_memory().unwrap();

    let mut memory_buf = memory.data(store);
    
    let input_size = outgoing.len() as u32;

    copy(outgoing.as_ptr(), memory_buf.as_ptr() as *mut u8, 1);

    Ok(())
}

// example function using the wasmtime api to allocate memory
unsafe fn allocateToWasmMemory(memory: Memory, store: &mut Store<()>) -> Result<(), Box<dyn Error>> {

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, WASM_BYTES_PATH)?;
    let mut store = Store::new(&engine, ());
    let mut instance = Instance::new(&mut store, &module, &[])?;

    // Load up our exports from the wasmtime instance
    let memory = instance.
        get_export(&mut store, "memory").unwrap()
        .into_memory().unwrap();

    let memory_buf = memory.data(&mut store);

    let add = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "add_wasm")
        .expect("`add_wasm` was not an exported function");

    // let allocate = instance.get_export(&mut store, "allocate").unwrap().into_func().unwrap();
    let allocate = instance
        .get_typed_func::<u32, u32>(&mut store, "allocate")
        .expect("`allocate` was not an exported function");

    let deallocate = instance
        .get_typed_func::<(u32, u32), ()>(&mut store, "deallocate")
        .expect("`deallocate` was not an exported function");

    let allocate_leaked_ptr = allocate.call(&mut store, BUF_SIZE)?;
    let res = memory.write(&mut store, allocate_leaked_ptr as usize, String::from("something2").as_bytes());
    
    // call read_string in wasm module
    let read_string = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "read_string")
        .expect("`read_string was not an exported function`");
    
    let ans = read_string.call(&mut store, (allocate_leaked_ptr, String::from("12345678").len() as u32))?;
    println!("wasm output: {ans}");
    
    unsafe{
        let mut temp_vec = vec![0u8; BUF_SIZE as usize];
        let slice = memory.read(&mut store, allocate_leaked_ptr as usize, &mut temp_vec).unwrap();
        let utf8 = std::str::from_utf8_unchecked(temp_vec.as_slice());
        println!("wasm output string: {0}", utf8);
    }

    let result = add.call(&mut store, (3, 8))?;
    println!("Answer: {:?}", result);
    Ok(())
}
