# Design decisions — Checkpoint 2

## D2.1 — Parallel compare lookup

First multi-page implementation uses O(`NUM_PAGES`) parallel comparison.
Indexed/direct-mapped lookup is deferred until correctness is proven.

## D2.2 — Lowest-index free slot

ALLOCATE picks the first invalid entry. Deterministic for differential testing.

## D2.3 — Miss is not error

Access to an untracked page sets `resp_miss=1` without `resp_error`. No eviction
or allocation is implied.

## D2.4 — ACCESS opcode

`OP_ACCESS` aliases READ/WRITE behavior. Keeps checkpoint 1 opcodes while
matching checkpoint 2 terminology.

## D2.5 — One event per commit

Single-event interface preserved from checkpoint 1. Future multi-event bus can
widen `event_valid` to a vector without changing table storage.

## D2.6 — Configurable NUM_PAGES

Default 16 for simulation speed. Override at build:

```bash
NUM_PAGES=64 make verilate
```

Rust `NUM_PAGES` constant must match the Verilated build.

## D2.7 — Metrics CSV for plotting

`checkpoint2_metrics.csv` records events, cycles, hits, misses, allocations,
releases for downstream analysis.
