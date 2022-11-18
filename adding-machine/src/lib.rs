// this is going to be running in wasmtime
// since this will be compiled to wasm32


// this is the guest rust program
extern crate alloc;
use std::slice;
use std::mem::MaybeUninit;
use alloc::vec::Vec;

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
