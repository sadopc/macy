#![allow(dead_code, deprecated)]

use libc::{
    c_int, host_processor_info, mach_host_self, mach_msg_type_number_t, natural_t,
    processor_info_array_t, PROCESSOR_CPU_LOAD_INFO,
};
use std::mem;

/// Per-CPU tick data.
#[derive(Debug, Clone, Default)]
struct CpuTicks {
    user: u64,
    system: u64,
    idle: u64,
    nice: u64,
}

impl CpuTicks {
    fn total(&self) -> u64 {
        self.user + self.system + self.idle + self.nice
    }

    fn active(&self) -> u64 {
        self.user + self.system + self.nice
    }
}

/// CPU usage tracker that computes deltas between samples.
pub struct CpuTracker {
    prev: Vec<CpuTicks>,
}

#[derive(Debug, Clone, Default)]
pub struct CpuUsage {
    pub overall_percent: f64,
    pub per_core: Vec<f64>,
}

impl CpuTracker {
    pub fn new() -> Self {
        let ticks = read_cpu_ticks();
        Self { prev: ticks }
    }

    /// Sample current CPU ticks and compute usage since last call.
    pub fn sample(&mut self) -> CpuUsage {
        let current = read_cpu_ticks();

        let mut per_core = Vec::with_capacity(current.len());
        let mut total_active: u64 = 0;
        let mut total_all: u64 = 0;

        for (curr, prev) in current.iter().zip(self.prev.iter()) {
            let d_active = curr.active().saturating_sub(prev.active());
            let d_total = curr.total().saturating_sub(prev.total());

            let usage = if d_total > 0 {
                d_active as f64 / d_total as f64 * 100.0
            } else {
                0.0
            };
            per_core.push(usage);

            total_active += d_active;
            total_all += d_total;
        }

        let overall = if total_all > 0 {
            total_active as f64 / total_all as f64 * 100.0
        } else {
            0.0
        };

        self.prev = current;

        CpuUsage {
            overall_percent: overall,
            per_core,
        }
    }
}

/// Read per-CPU tick counts via host_processor_info.
fn read_cpu_ticks() -> Vec<CpuTicks> {
    unsafe {
        let mut num_cpus: natural_t = 0;
        let mut cpu_info: processor_info_array_t = std::ptr::null_mut();
        let mut info_count: mach_msg_type_number_t = 0;

        let kr = host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO as c_int,
            &mut num_cpus,
            &mut cpu_info,
            &mut info_count,
        );

        if kr != 0 || cpu_info.is_null() {
            return Vec::new();
        }

        let mut ticks = Vec::with_capacity(num_cpus as usize);
        // Each CPU has CPU_STATE_MAX (4) entries
        for i in 0..num_cpus as isize {
            let base = i * 4; // CPU_STATE_MAX = 4
            ticks.push(CpuTicks {
                user: *cpu_info.offset(base) as u64,
                system: *cpu_info.offset(base + 1) as u64,
                idle: *cpu_info.offset(base + 2) as u64,
                nice: *cpu_info.offset(base + 3) as u64,
            });
        }

        // Deallocate the info array
        extern "C" {
            fn vm_deallocate(
                target_task: u32,
                address: usize,
                size: usize,
            ) -> i32;
            fn mach_task_self() -> u32;
        }
        vm_deallocate(
            mach_task_self(),
            cpu_info as usize,
            info_count as usize * mem::size_of::<i32>(),
        );

        ticks
    }
}
