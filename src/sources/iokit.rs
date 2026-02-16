#![allow(dead_code)]

use core_foundation::base::TCFType;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use std::ffi::c_void;

type IOReturn = i32;
type MachPort = u32;

#[allow(non_upper_case_globals)]
const kIOMainPortDefault: MachPort = 0;

extern "C" {
    fn IOServiceMatching(name: *const i8) -> *mut c_void;
    fn IOServiceGetMatchingService(mainPort: MachPort, matching: *mut c_void) -> u32;
    fn IORegistryEntryCreateCFProperty(
        entry: u32,
        key: *const c_void,
        allocator: *const c_void,
        options: u32,
    ) -> *const c_void;
    fn IOObjectRelease(object: u32) -> IOReturn;
}

/// Get an integer property from an IOKit registry entry matched by class name.
pub fn get_iokit_int_property(class_name: &str, property: &str) -> Option<i64> {
    unsafe {
        let class_cstr = std::ffi::CString::new(class_name).ok()?;
        let matching = IOServiceMatching(class_cstr.as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(kIOMainPortDefault, matching);
        if service == 0 {
            return None;
        }
        let key = CFString::new(property);
        let prop = IORegistryEntryCreateCFProperty(
            service,
            key.as_CFTypeRef(),
            std::ptr::null(),
            0,
        );
        IOObjectRelease(service);
        if prop.is_null() {
            return None;
        }
        let cf_num = CFNumber::wrap_under_create_rule(prop as *const _);
        cf_num.to_i64()
    }
}

/// Get a Data property from IOKit as raw bytes.
pub fn get_iokit_data_property(class_name: &str, property: &str) -> Option<Vec<u8>> {
    unsafe {
        let class_cstr = std::ffi::CString::new(class_name).ok()?;
        let matching = IOServiceMatching(class_cstr.as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(kIOMainPortDefault, matching);
        if service == 0 {
            return None;
        }
        let key = CFString::new(property);
        let prop = IORegistryEntryCreateCFProperty(
            service,
            key.as_CFTypeRef(),
            std::ptr::null(),
            0,
        );
        IOObjectRelease(service);
        if prop.is_null() {
            return None;
        }
        extern "C" {
            fn CFDataGetLength(data: *const c_void) -> isize;
            fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
            fn CFRelease(cf: *const c_void);
        }
        let len = CFDataGetLength(prop) as usize;
        let ptr = CFDataGetBytePtr(prop);
        let bytes = std::slice::from_raw_parts(ptr, len).to_vec();
        CFRelease(prop);
        Some(bytes)
    }
}

/// Get the GPU core count from IOKit AGXAccelerator.
pub fn gpu_core_count() -> Option<i64> {
    // Try AGXAccelerator first, fall back to AGXAcceleratorG13X etc.
    for class in &[
        "AGXAccelerator",
        "AGXAcceleratorG13X",
        "AGXAcceleratorG13G",
        "AGXAcceleratorG14X",
        "AGXAcceleratorG14G",
        "AGXAcceleratorG15X",
        "AGXAcceleratorG15G",
        "AGXAcceleratorG16X",
    ] {
        if let Some(count) = get_iokit_int_property(class, "gpu-core-count") {
            return Some(count);
        }
    }
    None
}

/// Get all IOKit properties for a service as a CFDictionary.
pub fn get_iokit_properties(class_name: &str) -> Option<CFMutableDictionary> {
    unsafe {
        let class_cstr = std::ffi::CString::new(class_name).ok()?;
        let matching = IOServiceMatching(class_cstr.as_ptr());
        if matching.is_null() {
            return None;
        }
        let service = IOServiceGetMatchingService(kIOMainPortDefault, matching);
        if service == 0 {
            return None;
        }
        extern "C" {
            fn IORegistryEntryCreateCFProperties(
                entry: u32,
                properties: *mut *const c_void,
                allocator: *const c_void,
                options: u32,
            ) -> IOReturn;
        }
        let mut props: *const c_void = std::ptr::null();
        let kr = IORegistryEntryCreateCFProperties(service, &mut props, std::ptr::null(), 0);
        IOObjectRelease(service);
        if kr != 0 || props.is_null() {
            return None;
        }
        Some(CFMutableDictionary::wrap_under_create_rule(props as *mut _))
    }
}
