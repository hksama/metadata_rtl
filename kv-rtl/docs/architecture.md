# System Architecture Overview

High-level stack for KV-RTL through **Checkpoint 3** (multi-page metadata table +
hotness policies).

For detail see:

- [rust_architecture.md](rust_architecture.md) — Rust modules and test harness
- [rtl_architecture.md](rtl_architecture.md) — SystemVerilog design
- [implementation_map.md](implementation_map.md) — Rust vs RTL vs not implemented
- [code_architecture.md](code_architecture.md) — documentation index

## Components

```text
Rust harness + MetadataTable + policy + trace_gen + eval + metrics
        │
        ▼
MetadataSimulator trait  (rust/src/lib.rs)
        │
        ▼
C ABI (sim/simulator.h, sim/simulator.cpp)
        │
        ▼
Verilator (top → kv_metadata_engine → kv_hotness_engine)
        │
        ▼
rtl/*.sv
```

## RTL modules (Checkpoint 3)

| Module | Role |
|--------|------|
| `kv_pkg` | Widths, opcodes, policy functions, saturation helpers |
| `kv_hotness_engine` | Per-entry hotness scoring (combinational) |
| `kv_metadata_engine` | Multi-page table, lookup, 2-cycle pipeline, aging |
| `top` | Parameterized wrapper |

## Rust modules (Checkpoint 3)

| Module | Role |
|--------|------|
| `types.rs` | Shared encodings (must match RTL) |
| `model.rs` | `MetadataTable` golden reference |
| `policy.rs` | Hotness policy mirror + hardware cost model |
| `trace_gen.rs` | Synthetic workloads |
| `oracle.rs` | Future-reuse ground truth (eval only) |
| `eval.rs` | Policy evaluation harness |
| `metrics.rs` | Counters, classification, ranking, CSV |
| `verilator.rs` | FFI to C simulator |

See [protocol.md](protocol.md) for timing and response semantics.  
See [hotness_policies.md](hotness_policies.md) for policy details.
