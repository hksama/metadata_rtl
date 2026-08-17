# KV Cache Management RTL — MVP Roadmap

## Project Goal

Build and iteratively refine a hardware-assisted KV-cache metadata and
placement-management engine using:

- SystemVerilog
- Verilator
- Rust
- cocotb and/or SystemVerilog testbenches
- synthesis tools later
- real/synthetic LLM access traces later

The ultimate research question is:

> Can a small specialized hardware engine accelerate KV-cache metadata
> management and placement decisions sufficiently to justify the
> hardware/software coordination overhead?

The project must NOT assume that hardware acceleration is beneficial.

The implementation must be designed so that the final evaluation can
demonstrate either:

1. hardware provides a meaningful benefit under certain workload regimes, or
2. hardware does not justify itself under realistic workloads.

Both outcomes are valid research results.

---

# Global Architecture Evolution

The implementation should evolve through the following stages:

```text
Checkpoint 0
Basic RTL development environment
        ↓
Checkpoint 1
Single-page metadata tracker
        ↓
Checkpoint 2
Multi-page KV metadata table
        ↓
Checkpoint 3
Hotness tracking policies
        ↓
Checkpoint 4
Candidate selection + eviction
        ↓
Checkpoint 5
Batched asynchronous metadata accelerator
        ↓
Checkpoint 6
Rust runtime integration
        ↓
Checkpoint 7
Real LLM/KV access traces
        ↓
Checkpoint 8
NUMA/tiered memory placement
        ↓
Checkpoint 9
Hardware/software coordination model
        ↓
Checkpoint 10
Synthesis + area/timing/power evaluation
        ↓
Checkpoint 11
Workload-specific optimization
        ↓
Checkpoint 12
Final research prototype
