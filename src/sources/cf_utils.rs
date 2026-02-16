#![allow(dead_code)]

use core_foundation::base::TCFType;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;

use super::CVoidRef;

/// Create a CFString from a Rust &str.
pub fn cfstr(s: &str) -> CFString {
    CFString::new(s)
}

/// Convert a CFString pointer to a Rust String.
/// Returns None if the pointer is null.
pub unsafe fn from_cfstring(ptr: CVoidRef) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cf = CFString::wrap_under_get_rule(ptr as *const _);
    Some(cf.to_string())
}

/// Get a value from a CFDictionary by string key.
pub unsafe fn cfdict_get_value(dict: CVoidRef, key: &str) -> CVoidRef {
    if dict.is_null() {
        return std::ptr::null();
    }
    extern "C" {
        fn CFDictionaryGetValue(dict: CVoidRef, key: CVoidRef) -> CVoidRef;
    }
    let cf_key = cfstr(key);
    CFDictionaryGetValue(dict, cf_key.as_CFTypeRef())
}

/// Read an i64 from a CFNumber pointer.
pub unsafe fn cfnum_to_i64(ptr: CVoidRef) -> Option<i64> {
    if ptr.is_null() {
        return None;
    }
    let cf_num = CFNumber::wrap_under_get_rule(ptr as *const _);
    cf_num.to_i64()
}

/// Read an f64 from a CFNumber pointer.
pub unsafe fn cfnum_to_f64(ptr: CVoidRef) -> Option<f64> {
    if ptr.is_null() {
        return None;
    }
    let cf_num = CFNumber::wrap_under_get_rule(ptr as *const _);
    cf_num.to_f64()
}

/// Get a string value from a CFDictionary by key.
pub unsafe fn cfdict_get_string(dict: CVoidRef, key: &str) -> Option<String> {
    let val = cfdict_get_value(dict, key);
    from_cfstring(val)
}

/// Get an i64 value from a CFDictionary by key.
pub unsafe fn cfdict_get_i64(dict: CVoidRef, key: &str) -> Option<i64> {
    let val = cfdict_get_value(dict, key);
    cfnum_to_i64(val)
}

/// Get the count of a CFArray.
pub unsafe fn cfarray_count(arr: CVoidRef) -> isize {
    if arr.is_null() {
        return 0;
    }
    extern "C" {
        fn CFArrayGetCount(arr: CVoidRef) -> isize;
    }
    CFArrayGetCount(arr)
}

/// Get a value at index from a CFArray.
pub unsafe fn cfarray_get(arr: CVoidRef, idx: isize) -> CVoidRef {
    if arr.is_null() {
        return std::ptr::null();
    }
    extern "C" {
        fn CFArrayGetValueAtIndex(arr: CVoidRef, idx: isize) -> CVoidRef;
    }
    CFArrayGetValueAtIndex(arr, idx)
}
