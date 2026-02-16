use crate::sources::iokit;
use std::mem;

/// Static information about the SoC.
#[derive(Debug, Clone)]
pub struct SocInfo {
    pub chip_name: String,
    pub e_cores: u32,
    pub p_cores: u32,
    pub gpu_cores: u32,
    pub gpu_freqs: Vec<u32>,  // MHz, sorted ascending (P1..P15)
    pub total_memory_gb: f64,
}

impl SocInfo {
    #[allow(dead_code)]
    pub fn total_cpu_cores(&self) -> u32 {
        self.e_cores + self.p_cores
    }
}

impl std::fmt::Display for SocInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}E+{}P CPU, {} GPU, {:.0}GB)",
            self.chip_name,
            self.e_cores,
            self.p_cores,
            self.gpu_cores,
            self.total_memory_gb
        )
    }
}

/// Read a sysctl value as a string.
fn sysctl_string(name: &str) -> Option<String> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut size: usize = 0;
    unsafe {
        libc::sysctlbyname(cname.as_ptr(), std::ptr::null_mut(), &mut size, std::ptr::null_mut(), 0);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size];
        libc::sysctlbyname(
            cname.as_ptr(),
            buf.as_mut_ptr() as *mut _,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        // Remove trailing null
        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            buf.truncate(pos);
        }
        String::from_utf8(buf).ok()
    }
}

/// Read a sysctl value as u32.
fn sysctl_u32(name: &str) -> Option<u32> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u32 = 0;
    let mut size = mem::size_of::<u32>();
    unsafe {
        let kr = libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u32 as *mut _,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if kr == 0 { Some(val) } else { None }
    }
}

/// Read a sysctl value as u64.
fn sysctl_u64(name: &str) -> Option<u64> {
    let cname = std::ffi::CString::new(name).ok()?;
    let mut val: u64 = 0;
    let mut size = mem::size_of::<u64>();
    unsafe {
        let kr = libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut u64 as *mut _,
            &mut size,
            std::ptr::null_mut(),
            0,
        );
        if kr == 0 { Some(val) } else { None }
    }
}

/// Try to read GPU frequency table from IOKit, fall back to known tables.
fn detect_gpu_freqs(chip_name: &str) -> Vec<u32> {
    // Try to get GPU frequencies from IOKit AGXAccelerator properties
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
        if let Some(data) = iokit::get_iokit_data_property(class, "gpu-perf-state-mapped-frequencies") {
            // Data is an array of u32 frequencies in Hz, stored as little-endian
            let freqs: Vec<u32> = data
                .chunks_exact(4)
                .map(|chunk| {
                    u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) / 1_000_000
                })
                .filter(|&f| f > 0)
                .collect();
            if !freqs.is_empty() {
                return freqs;
            }
        }
    }

    // Fallback to known frequency tables by chip
    if chip_name.contains("M4 Pro") || chip_name.contains("M4 Max") {
        vec![444, 612, 808, 968, 1110, 1236, 1338, 1398]
    } else if chip_name.contains("M4") {
        vec![396, 528, 660, 792, 924, 1056, 1164, 1290, 1398]
    } else if chip_name.contains("M3") {
        vec![444, 612, 808, 968, 1110, 1236, 1338, 1398]
    } else if chip_name.contains("M2") {
        vec![396, 528, 660, 792, 924, 1056, 1164, 1290, 1398]
    } else if chip_name.contains("M1") {
        vec![396, 528, 660, 792, 924, 1056, 1164, 1278]
    } else {
        vec![396, 528, 660, 792, 924, 1056, 1164, 1290, 1398]
    }
}

/// Detect the SoC info from the current system.
pub fn detect() -> SocInfo {
    let chip_name = sysctl_string("machdep.cpu.brand_string")
        .unwrap_or_else(|| "Unknown".to_string());

    // P-cores = perflevel0, E-cores = perflevel1
    let p_cores = sysctl_u32("hw.perflevel0.logicalcpu").unwrap_or(4);
    let e_cores = sysctl_u32("hw.perflevel1.logicalcpu").unwrap_or(6);

    let gpu_cores = iokit::gpu_core_count().unwrap_or(10) as u32;

    // Get GPU frequency table from IOKit (gpu-perf-state-freqs or similar)
    let gpu_freqs = detect_gpu_freqs(&chip_name);

    let total_mem = sysctl_u64("hw.memsize").unwrap_or(16 * 1024 * 1024 * 1024);
    let total_memory_gb = total_mem as f64 / (1024.0 * 1024.0 * 1024.0);

    SocInfo {
        chip_name,
        e_cores,
        p_cores,
        gpu_cores,
        gpu_freqs,
        total_memory_gb,
    }
}
