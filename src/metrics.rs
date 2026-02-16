use crate::soc::SocInfo;
use crate::sources::cpu::{CpuTracker, CpuUsage};
use crate::sources::ioreport::{self, GpuMetrics, IOReportSubscription, PowerMetrics};
use crate::sources::memory::{self, MemoryInfo};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// All metrics collected in one sample.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub cpu: CpuUsage,
    pub gpu: GpuMetrics,
    pub memory: MemoryInfo,
    pub power: PowerMetrics,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            cpu: CpuUsage::default(),
            gpu: GpuMetrics::default(),
            memory: MemoryInfo::default(),
            power: PowerMetrics::default(),
        }
    }
}

/// Start the background sampler thread. Returns a receiver for metrics.
pub fn start_sampler(interval: Duration, soc: SocInfo) -> mpsc::Receiver<Metrics> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        sampler_loop(tx, interval, &soc);
    });

    rx
}

fn sampler_loop(tx: mpsc::Sender<Metrics>, interval: Duration, soc: &SocInfo) {
    let mut cpu_tracker = CpuTracker::new();

    // Subscribe to IOReport channels for GPU stats and energy
    let subscription = ioreport::create_subscription(&["GPU Stats", "Energy Model"]);

    // Initial sleep to let the first CPU tick delta accumulate
    thread::sleep(interval);

    loop {
        let metrics = match &subscription {
            Some(sub) => sample_with_ioreport(sub, &mut cpu_tracker, interval, &soc.gpu_freqs),
            None => sample_without_ioreport(&mut cpu_tracker),
        };

        if tx.send(metrics).is_err() {
            break;
        }
    }
}

fn sample_with_ioreport(
    sub: &IOReportSubscription,
    cpu_tracker: &mut CpuTracker,
    interval: Duration,
    gpu_freqs: &[u32],
) -> Metrics {
    let s1 = ioreport::create_sample(sub);
    let t1 = Instant::now();

    thread::sleep(interval);

    let s2 = ioreport::create_sample(sub);
    let elapsed_ns = t1.elapsed().as_nanos() as u64;

    let delta = ioreport::create_delta(s1, s2);
    ioreport::release_sample(s1);
    ioreport::release_sample(s2);

    let (gpu_util, gpu_freq) = ioreport::parse_gpu_stats(delta, gpu_freqs);
    let power = ioreport::parse_power(delta, elapsed_ns);

    ioreport::release_sample(delta);

    let cpu = cpu_tracker.sample();
    let mem = memory::get_memory_info();

    Metrics {
        cpu,
        gpu: GpuMetrics {
            utilization: gpu_util,
            freq_mhz: gpu_freq,
            power_watts: power.gpu_watts,
        },
        memory: mem,
        power,
    }
}

fn sample_without_ioreport(cpu_tracker: &mut CpuTracker) -> Metrics {
    let cpu = cpu_tracker.sample();
    let mem = memory::get_memory_info();

    Metrics {
        cpu,
        gpu: GpuMetrics::default(),
        memory: mem,
        power: PowerMetrics::default(),
    }
}
