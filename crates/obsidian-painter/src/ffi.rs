//! The WebAssembly boundary: strings in and out through linear memory.
//!
//! The host allocates with [`painter_alloc`], writes UTF-8, calls
//! [`painter_call`], reads the answer at the returned `ptr << 32 | len`, and
//! frees both with [`painter_free`]. The answer is `{"ok": ...}` or
//! `{"error": "..."}`.
#![allow(unsafe_code)]

#[no_mangle]
pub extern "C" fn painter_alloc(len: usize) -> *mut u8 {
    // a boxed slice has exactly `len` bytes, so `painter_free` can rebuild it
    Box::into_raw(vec![0_u8; len].into_boxed_slice()).cast::<u8>()
}

/// # Safety
/// `ptr` and `len` must come from [`painter_alloc`] or [`painter_call`].
#[no_mangle]
pub unsafe extern "C" fn painter_free(ptr: *mut u8, len: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
}

unsafe fn text<'a>(ptr: *const u8, len: usize) -> &'a str {
    std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap_or("")
}

/// # Safety
/// The pointers must point at `len` bytes the host wrote.
#[no_mangle]
pub unsafe extern "C" fn painter_call(
    op: *const u8,
    op_len: usize,
    input: *const u8,
    input_len: usize,
) -> u64 {
    let answer = match crate::call(text(op, op_len), text(input, input_len)) {
        Ok(v) => serde_json::json!({ "ok": v }),
        Err(e) => serde_json::json!({ "error": e }),
    };
    let bytes = answer.to_string().into_bytes().into_boxed_slice();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes).cast::<u8>();
    ((ptr as u64) << 32) | len as u64
}
