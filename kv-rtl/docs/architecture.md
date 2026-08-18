# Checkpoint 2 — Architecture

## Components

```text
Rust harness + MetadataTable + trace_gen + metrics
        │
        ▼
MetadataSimulator trait
        │
        ▼
C ABI (sim/simulator.h)
        │
        ▼
Verilator (top → kv_metadata_engine)
        │
        ▼
rtl/kv_metadata_engine.sv  (NUM_PAGES-entry table)
```

## RTL modules

| Module | Role |
|--------|------|
| `kv_pkg` | Widths, opcodes, saturation helpers, `DEFAULT_NUM_PAGES` |
| `kv_metadata_engine` | Parallel-lookup metadata table |
| `top` | Parameterized wrapper (`NUM_PAGES` overridable at Verilate time) |

## Lookup

On commit, all `NUM_PAGES` entries are compared to `page_id` in parallel.
First match wins. Free-slot search scans for `!valid` (lowest index first).

## Rust modules

| Module | Role |
|--------|------|
| `model.rs` | `MetadataTable` golden reference |
| `trace_gen.rs` | Synthetic workloads (hot/warm/cold, sequential, random, bursty) |
| `metrics.rs` | Cycle/hit/miss counters + CSV export |

See [protocol.md](protocol.md) for timing and response semantics.
