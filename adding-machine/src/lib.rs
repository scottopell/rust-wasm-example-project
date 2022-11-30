// this is going to be running in wasmtime
// since this will be compiled to wasm32


// this is the guest rust program
extern crate alloc;
use std::slice;
use std::mem::MaybeUninit;
use alloc::vec::Vec;

use ::value::Value;
use value::Secrets;
use vrl::diagnostic::DiagnosticList;
use vrl::state::TypeState;
use vrl::{diagnostic::Formatter, prelude::BTreeMap, CompileConfig, Runtime};
use vrl::{TargetValue, Terminate, TimeZone};


// maybe not needed but will be the the formatted output glue
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub program: String,
    pub event: Value,
}

impl Input {
    pub fn new(program: &str, event: Value) -> Self {
        Self {
            program: program.to_owned(),
            event,
        }
    }
}

// The module returns the result of the last expression and the event that results from the
// applied program
#[derive(Deserialize, Serialize)]
pub struct VrlCompileResult {
    pub output: Value,
    pub result: Value,
}

impl VrlCompileResult {
    fn new(output: Value, result: Value) -> Self {
        Self { output, result }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct VrlDiagnosticResult {
    pub list: Vec<String>,
    pub msg: String,
    pub msg_colorized: String,
}

impl VrlDiagnosticResult {
    fn new(program: &str, diagnostic_list: DiagnosticList) -> Self {
        Self {
            list: diagnostic_list
                .clone()
                .into_iter()
                .map(|diag| String::from(diag.message()))
                .collect(),
            msg: Formatter::new(program, diagnostic_list.clone()).to_string(),
            msg_colorized: Formatter::new(program, diagnostic_list)
                .colored()
                .to_string(),
        }
    }

    fn new_runtime_error(program: &str, terminate: Terminate) -> Self {
        Self {
            list: Vec::with_capacity(1),
            msg: Formatter::new(program, terminate.clone().get_expression_error()).to_string(),
            msg_colorized: Formatter::new(program, terminate.get_expression_error())
                .colored()
                .to_string(),
        }
    }
}

fn compile(mut input: Input) -> Result<VrlCompileResult, VrlDiagnosticResult> {
    let event = &mut input.event;
    let functions = stdlib::all();
    let state = TypeState::default();
    let mut runtime = Runtime::default();
    let config = CompileConfig::default();
    let timezone = TimeZone::default();

    let mut target_value = TargetValue {
        value: event.clone(),
        metadata: Value::Object(BTreeMap::new()),
        secrets: Secrets::new(),
    };

    let program = match vrl::compile_with_state(&input.program, &functions, &state, config) {
        Ok(program) => program,
        Err(diagnostics) => return Err(VrlDiagnosticResult::new(&input.program, diagnostics)),
    };

    match runtime.resolve(&mut target_value, &program.program, &timezone) {
        Ok(result) => Ok(VrlCompileResult::new(result, target_value.value)),
        Err(err) => Err(VrlDiagnosticResult::new_runtime_error(&input.program, err)),
    }
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "run_vrl_wasm")]
#[no_mangle]
pub unsafe extern "C" fn run_vrl(ptr: u32, len: u32) -> u32 {
    let incoming = &ptr_to_string(ptr, len);
    
    let input: Input = serde_json::from_str(incoming).unwrap();

    match compile(input) {
        Ok(res) => {
            let res_json_str = serde_json::to_value(res).unwrap().to_string();

            store_string_at_ptr(&res_json_str, ptr);
            res_json_str.len() as u32
        },
        Err(err) => {
            let err_json_str = serde_json::to_value(err).unwrap().to_string();
            store_string_at_ptr(&err_json_str, ptr);
            err_json_str.len() as u32
        }
    }
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "add_wasm")]
#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "echo_string")]
#[no_mangle]
pub unsafe extern "C" fn echo_string(ptr: u32, len: u32) -> u32 {
    ptr + len
}

// this will read a string that was written
// by the runrunrun/src/main.rs file
// returns the length of the string read
#[cfg_attr(all(target_arch = "wasm32"), export_name = "read_string")]
#[no_mangle]
pub unsafe extern "C" fn read_string(ptr: u32, len: u32) -> u32 {
    let incoming_string = &ptr_to_string(ptr, len);
    incoming_string.len() as u32
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "return_string")]
#[no_mangle]
pub unsafe extern "C" fn return_string(ptr: u32, len: u32) -> u32 {
    let incoming_string = &ptr_to_string(ptr, len);
    let new_string = format!("Incoming: {incoming_string}\nThis string was written from adding-machine/src/lib.rs");
    store_string_at_ptr(&new_string, ptr);

    new_string.len() as u32
}

// WASM Memory-related helper functinos
//
// TODO explore using lol_alloc instead of default rust allocator
/// WebAssembly export that allocates a pointer (linear memory offset) that can
/// be used for a string.
///
/// This is an ownership transfer, which means the caller must call
/// [`deallocate`] when finished.
#[cfg_attr(all(target_arch = "wasm32"), export_name = "allocate")]
#[no_mangle]
pub extern "C" fn _allocate(size: u32) -> *mut u8 {
    allocate(size as usize)
}

/// Allocates size bytes and leaks the pointer where they start.
fn allocate(size: usize) -> *mut u8 {
    // Allocate the amount of bytes needed.
    let vec: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);

    // into_raw leaks the memory to the caller.
    Box::into_raw(vec.into_boxed_slice()) as *mut u8
}

/// WebAssembly export that deallocates a pointer of the given size (linear
/// memory offset, byteCount) allocated by [`allocate`].
#[cfg_attr(all(target_arch = "wasm32"), export_name = "deallocate")]
#[no_mangle]
pub unsafe extern "C" fn _deallocate(ptr: u32, size: u32) {
    deallocate(ptr as *mut u8, size as usize);
}

/// Retakes the pointer which allows its memory to be freed.
unsafe fn deallocate(ptr: *mut u8, size: usize) {
    // TODO - should this be Box::from_raw? (see Box::into_raw docs)
    let _ = Vec::from_raw_parts(ptr, 0, size);
}



// WASM String-related helper functions
/// Returns a string from WebAssembly compatible numeric types representing
/// its pointer and length.
unsafe fn ptr_to_string(ptr: u32, len: u32) -> String {
    let slice = slice::from_raw_parts_mut(ptr as *mut u8, len as usize);
    let utf8 = std::str::from_utf8_unchecked_mut(slice);
    return String::from(utf8);
}


/// Stores the given string 's' at the memory location pointed to by 'ptr'
/// This assumes no buffer overflows - here be dragons.
unsafe fn store_string_at_ptr(s: &str, ptr: u32) {
    // Create a mutable slice of u8 pointing at the buffer given as 'ptr'
    // with a length of the string we're about to copy into it
    let dest = slice::from_raw_parts_mut(ptr as *mut u8, s.len() as usize);
    dest.copy_from_slice(s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
