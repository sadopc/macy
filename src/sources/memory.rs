#![allow(deprecated)]

use libc::{
    c_int, host_statistics64, mach_host_self, mach_msg_type_number_t,
    vm_statistics64, HOST_VM_INFO64, HOST_VM_INFO64_COUNT,
};
use std::mem;

#[derive(Debug, Clone, Default)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
}

impl MemoryInfo {
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn used_gb(&self) -> f64 {
        self.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn usage_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.used_bytes as f64 / self.total_bytes as f64 * 100.0
    }
}

/// Get total physical memory via sysctl hw.memsize.
fn total_memory() -> u64 {
    let mut size: u64 = 0;
    let mut len = mem::size_of::<u64>();
    let name = c"hw.memsize";
    unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            &mut size as *mut u64 as *mut _,
            &mut len,
            std::ptr::null_mut(),
            0,
        );
    }
    size
}

/// Get current memory usage via host_statistics64.
pub fn get_memory_info() -> MemoryInfo {
    let total = total_memory();

    let mut vm_stat: vm_statistics64 = unsafe { mem::zeroed() };
    let mut count: mach_msg_type_number_t = HOST_VM_INFO64_COUNT as _;

    let kr = unsafe {
        host_statistics64(
            mach_host_self(),
            HOST_VM_INFO64 as c_int,
            &mut vm_stat as *mut vm_statistics64 as *mut _,
            &mut count,
        )
    };

    if kr != 0 {
        return MemoryInfo {
            total_bytes: total,
            used_bytes: 0,
        };
    }

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 };

    // Used = total - free - purgeable - (external pages that can be reclaimed)
    // This matches Activity Monitor's calculation
    let internal = vm_stat.internal_page_count as u64;
    let purgeable = vm_stat.purgeable_count as u64;
    let wired = vm_stat.wire_count as u64;
    let compressor = vm_stat.compressor_page_count as u64;

    // "App Memory" + wired + compressor is what Activity Monitor reports as "Memory Used"
    let app_memory = internal - purgeable;
    let used = (app_memory + wired + compressor) * page_size;

    MemoryInfo {
        total_bytes: total,
        used_bytes: used.min(total),
    }
}
