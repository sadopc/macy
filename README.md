# macy

A real-time TUI system monitor for macOS Apple Silicon. No sudo required.

```
┌─ Apple M4 (6E+4P CPU, 10 GPU, 16GB) ──────────────────── macy v0.1.0 ─┐
│                                                                          │
│  ┌─ CPU  47% ────────────────────┐  ┌─ GPU  23% @ 720 MHz ───────────┐ │
│  │ ▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▇█▇▅▃▃▅▇█▇▅▃ │  │ ▁▁▁▂▃▃▂▁▁▁▂▃▂▁▁▁▂▂▃▃▂▁▁▁▂▃▂ │ │
│  └───────────────────────────────┘  └─────────────────────────────────┘ │
│                                                                          │
│  ┌─ Memory  9.4 / 16.0 GB ──────┐  ┌─ Power ────────────────────────┐ │
│  │ ▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▇█▇▅▃▃▅▇█▇▅▃ │  │ CPU: 2.3W   GPU: 0.9W        │ │
│  └───────────────────────────────┘  │ Total: 3.2W                    │ │
│                                     │ ▁▂▃▅▇▅▃▂▁▁▂▃▅▇▅▃▃▅▇▅▃▂▁▁▂▃▅ │ │
│                                     └─────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────┤
│  q: quit                                                    interval 1s │
└──────────────────────────────────────────────────────────────────────────┘
```

## Features

- **CPU usage** — overall percentage via mach kernel tick deltas
- **GPU utilization & frequency** — DVFS residency from IOReport `GPUPH` channel
- **Memory** — used/total via `host_statistics64` + `sysctl hw.memsize`
- **Power** — CPU and GPU wattage from IOReport Energy Model channels
- **Sparkline history** — rolling 120-sample graphs for all metrics
- **852 KB binary** — single static release build, no runtime deps

## Requirements

- macOS on Apple Silicon (M1/M2/M3/M4)
- Rust 1.75+ (uses `c""` literal syntax)

## Install

```sh
cargo install --path .
```

Or build manually:

```sh
cargo build --release
./target/release/macy
```

## Usage

```
macy              # launch TUI dashboard (1s interval)
macy -i 500       # 500ms sampling interval
macy --print      # print metrics to stdout (no TUI)
```

**Controls:** `q` or `Esc` to quit.

## How it works

| Metric | Source | API |
|--------|--------|-----|
| GPU utilization | IOReport `GPU Stats/GPU Performance States` | DVFS residency ratio |
| GPU frequency | IOReport GPUPH channel | Weighted average of active P-states |
| GPU power | IOReport `Energy Model/GPU Energy` | Energy delta (nJ) / time |
| CPU usage | `host_processor_info()` | Tick deltas (user+sys / total) |
| CPU power | IOReport `Energy Model/CPU Energy` | Energy delta (mJ) / time |
| Memory | `host_statistics64()` + `sysctl hw.memsize` | VM page stats |
| Chip info | `sysctl machdep.cpu.brand_string` | |
| Core counts | `sysctl hw.perflevel{0,1}.logicalcpu` | P-cores / E-cores |
| GPU cores | IOKit `AGXAccelerator` registry | `gpu-core-count` |

All data sources work without sudo.

## License

MIT
