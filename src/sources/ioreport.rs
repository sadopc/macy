#![allow(dead_code)]

use super::cf_utils::*;
use super::CVoidRef;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use std::ffi::c_void;

// IOReport FFI - links against libIOReport.dylib (private framework, available on all macOS)
#[link(name = "IOReport")]
extern "C" {
    // Returns CFDictionaryRef containing "IOReportChannels" key
    fn IOReportCopyChannelsInGroup(
        group: CVoidRef,    // CFStringRef
        subgroup: CVoidRef, // CFStringRef or NULL
        a: u64,
        b: u64,
        c: u64,
    ) -> CVoidRef; // CFDictionaryRef

    // Merges channels from src into dst dictionary
    fn IOReportMergeChannels(
        dst: CVoidRef,  // CFDictionaryRef
        src: CVoidRef,  // CFDictionaryRef
        nil: CVoidRef,
    );

    fn IOReportCreateSubscription(
        a: CVoidRef,
        channels: CVoidRef, // CFMutableDictionaryRef
        out: *mut CVoidRef, // *mut CFMutableDictionaryRef
        d: u64,
        e: CVoidRef,
    ) -> CVoidRef; // IOReportSubscriptionRef

    fn IOReportCreateSamples(
        subscription: CVoidRef, // IOReportSubscriptionRef
        channels: CVoidRef,     // CFMutableDictionaryRef
        b: CVoidRef,
    ) -> CVoidRef; // CFDictionaryRef

    fn IOReportCreateSamplesDelta(
        s1: CVoidRef, // CFDictionaryRef
        s2: CVoidRef, // CFDictionaryRef
        a: CVoidRef,
    ) -> CVoidRef; // CFDictionaryRef

    fn IOReportChannelGetGroup(channel: CVoidRef) -> CVoidRef;
    fn IOReportChannelGetSubGroup(channel: CVoidRef) -> CVoidRef;
    fn IOReportChannelGetChannelName(channel: CVoidRef) -> CVoidRef;
    fn IOReportChannelGetUnitLabel(channel: CVoidRef) -> CVoidRef;
    fn IOReportStateGetCount(channel: CVoidRef) -> i32;
    fn IOReportStateGetNameForIndex(channel: CVoidRef, idx: i32) -> CVoidRef;
    fn IOReportStateGetResidency(channel: CVoidRef, idx: i32) -> i64;
    fn IOReportSimpleGetIntegerValue(channel: CVoidRef, a: i32) -> i64;
}

extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFDictionaryGetCount(dict: CVoidRef) -> isize;
    fn CFDictionaryCreateMutableCopy(
        allocator: CVoidRef,
        capacity: isize,
        dict: CVoidRef,
    ) -> CVoidRef;
}

// kCFAllocatorDefault
const CF_ALLOCATOR_DEFAULT: CVoidRef = std::ptr::null();

/// Subscription handle for IOReport sampling.
pub struct IOReportSubscription {
    subscription: CVoidRef,
    channels: CVoidRef, // CFMutableDictionaryRef
}

/// Parsed GPU metrics from IOReport delta.
#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    pub utilization: f64,
    pub freq_mhz: f64,
    pub power_watts: f64,
}

/// Parsed CPU power from IOReport delta.
#[derive(Debug, Clone, Default)]
pub struct PowerMetrics {
    pub cpu_watts: f64,
    pub gpu_watts: f64,
}

/// Get the channels array from a sample/delta CFDictionary.
unsafe fn get_channels_array(dict: CVoidRef) -> CVoidRef {
    cfdict_get_value(dict, "IOReportChannels")
}

/// Create a subscription for the given IOReport groups.
pub fn create_subscription(groups: &[&str]) -> Option<IOReportSubscription> {
    unsafe {
        let mut channel_dicts: Vec<CVoidRef> = Vec::new();

        for group in groups {
            let cf_group = CFString::new(group);
            let chan = IOReportCopyChannelsInGroup(
                cf_group.as_CFTypeRef(),
                std::ptr::null(),
                0,
                0,
                0,
            );
            if !chan.is_null() {
                channel_dicts.push(chan);
            }
        }

        if channel_dicts.is_empty() {
            return None;
        }

        // Merge all channel dicts into the first one
        let merged = channel_dicts[0];
        for i in 1..channel_dicts.len() {
            IOReportMergeChannels(merged, channel_dicts[i], std::ptr::null());
        }

        // Create a mutable copy for the subscription
        let size = CFDictionaryGetCount(merged);
        let mutable_channels = CFDictionaryCreateMutableCopy(CF_ALLOCATOR_DEFAULT, size, merged);

        // Release original channel dicts
        for dict in &channel_dicts {
            CFRelease(*dict);
        }

        if mutable_channels.is_null() {
            return None;
        }

        let mut out: CVoidRef = std::ptr::null();
        let subscription = IOReportCreateSubscription(
            std::ptr::null(),
            mutable_channels,
            &mut out,
            0,
            std::ptr::null(),
        );

        if subscription.is_null() {
            CFRelease(mutable_channels);
            return None;
        }

        Some(IOReportSubscription {
            subscription,
            channels: mutable_channels,
        })
    }
}

/// Take a snapshot sample.
pub fn create_sample(sub: &IOReportSubscription) -> CVoidRef {
    unsafe { IOReportCreateSamples(sub.subscription, sub.channels, std::ptr::null()) }
}

/// Compute delta between two samples.
pub fn create_delta(s1: CVoidRef, s2: CVoidRef) -> CVoidRef {
    unsafe { IOReportCreateSamplesDelta(s1, s2, std::ptr::null()) }
}

/// Release a sample/delta.
pub fn release_sample(sample: CVoidRef) {
    if !sample.is_null() {
        unsafe { CFRelease(sample) }
    }
}

/// Parse GPU utilization and frequency from a delta sample.
/// Uses GPUPH channel from "GPU Stats"/"GPU Performance States".
/// States are: OFF, P1, P2, ... P15 where P-states map to frequency levels.
/// gpu_freqs should be sorted ascending (lowest freq first) matching P1, P2, ...
pub fn parse_gpu_stats(delta: CVoidRef, gpu_freqs: &[u32]) -> (f64, f64) {
    unsafe {
        let items = get_channels_array(delta);
        if items.is_null() {
            return (0.0, 0.0);
        }

        let count = cfarray_count(items);
        let mut total_active: i64 = 0;
        let mut total: i64 = 0;
        let mut weighted_freq: f64 = 0.0;

        for i in 0..count {
            let ch = cfarray_get(items, i);
            if ch.is_null() {
                continue;
            }

            let group = from_cfstring(IOReportChannelGetGroup(ch));
            let subgroup = from_cfstring(IOReportChannelGetSubGroup(ch));
            let channel_name = from_cfstring(IOReportChannelGetChannelName(ch));

            let is_gpuph = group.as_deref() == Some("GPU Stats")
                && subgroup.as_deref() == Some("GPU Performance States")
                && channel_name.as_deref() == Some("GPUPH");

            if !is_gpuph {
                continue;
            }

            let state_count = IOReportStateGetCount(ch);
            for s in 0..state_count {
                let name = from_cfstring(IOReportStateGetNameForIndex(ch, s))
                    .unwrap_or_default();
                let residency = IOReportStateGetResidency(ch, s);

                total += residency;

                // OFF state = GPU is completely off
                if name == "OFF" {
                    continue;
                }

                // Active P-states: P1, P2, ..., P15
                // P-state index maps to gpu_freqs array (P1 -> freqs[0], P2 -> freqs[1], etc.)
                total_active += residency;

                if let Some(idx) = name.strip_prefix('P').and_then(|n| n.parse::<usize>().ok()) {
                    if idx > 0 && idx <= gpu_freqs.len() {
                        weighted_freq += gpu_freqs[idx - 1] as f64 * residency as f64;
                    }
                }
            }
        }

        let utilization = if total > 0 {
            (total_active as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let avg_freq = if total_active > 0 {
            weighted_freq / total_active as f64
        } else {
            0.0
        };

        (utilization, avg_freq)
    }
}

/// Convert energy value to joules based on unit string.
fn energy_to_joules(value: i64, unit: &str) -> f64 {
    match unit {
        "nJ" => value as f64 / 1_000_000_000.0,
        "uJ" => value as f64 / 1_000_000.0,
        "mJ" => value as f64 / 1_000.0,
        _ => value as f64 / 1_000_000_000.0, // default to nJ
    }
}

/// Parse power metrics from a delta sample.
pub fn parse_power(delta: CVoidRef, interval_ns: u64) -> PowerMetrics {
    unsafe {
        let items = get_channels_array(delta);
        if items.is_null() {
            return PowerMetrics::default();
        }

        let count = cfarray_count(items);
        let mut cpu_joules: f64 = 0.0;
        let mut gpu_joules: f64 = 0.0;

        for i in 0..count {
            let ch = cfarray_get(items, i);
            if ch.is_null() {
                continue;
            }

            let group = from_cfstring(IOReportChannelGetGroup(ch));
            if group.as_deref() != Some("Energy Model") {
                continue;
            }

            let channel_name = from_cfstring(IOReportChannelGetChannelName(ch))
                .unwrap_or_default();
            let unit = from_cfstring(IOReportChannelGetUnitLabel(ch))
                .unwrap_or_default();
            let value = IOReportSimpleGetIntegerValue(ch, 0);

            if channel_name == "GPU Energy" {
                gpu_joules += energy_to_joules(value, &unit);
            } else if channel_name == "CPU Energy" {
                cpu_joules += energy_to_joules(value, &unit);
            }
        }

        let interval_s = interval_ns as f64 / 1_000_000_000.0;

        let cpu_watts = if interval_s > 0.0 {
            cpu_joules / interval_s
        } else {
            0.0
        };
        let gpu_watts = if interval_s > 0.0 {
            gpu_joules / interval_s
        } else {
            0.0
        };

        PowerMetrics {
            cpu_watts,
            gpu_watts,
        }
    }
}
