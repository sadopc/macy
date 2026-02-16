pub mod cf_utils;
pub mod cpu;
pub mod iokit;
pub mod ioreport;
pub mod memory;

use std::ffi::c_void;

pub type CVoidRef = *const c_void;
