use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Convert LaTeX string to Unicode.
/// Caller is responsible for freeing the returned string using `latex_unicode_free`.
#[no_mangle]
pub extern "C" fn latex_to_unicode(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let r_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = latex_to_unicode::latex_to_unicode(r_str);
    match CString::new(result) {
        Ok(c_res) => c_res.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert LaTeX display block to 2D Unicode.
#[no_mangle]
pub extern "C" fn latex_to_unicode_block(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let r_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = latex_to_unicode::latex_to_unicode_block(r_str);
    match CString::new(result) {
        Ok(c_res) => c_res.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Transform entire markdown document.
#[no_mangle]
pub extern "C" fn latex_transform_markdown(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let r_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = latex_to_unicode::transform_markdown(r_str);
    match CString::new(result) {
        Ok(c_res) => c_res.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free memory allocated by this library.
#[no_mangle]
pub extern "C" fn latex_unicode_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
