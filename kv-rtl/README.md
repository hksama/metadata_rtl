# KV-RTL — Checkpoint 2

Multi-page KV metadata table with parallel lookup, hit/miss reporting,
synthetic workload generation, and metrics CSV output.

## Quick start

```bash
cd kv-rtl
make test
```

## Configure table size

```bash
NUM_PAGES=64 make verilate
make test
```

Rust default is `NUM_PAGES=16` in `rust/src/types.rs` — must match Verilator build.

## Outputs

| Artifact | Description |
|----------|-------------|
| `traces/synthetic/checkpoint2_trace.txt` | Sample workload (`A` alloc, `C` access, `R` release) |
| `traces/synthetic/checkpoint2_metrics.csv` | events, cycles, hits, misses, … |
| `traces/synthetic/checkpoint2.vcd` | Waveform from `make simulate` |

## Waveforms

```bash
make simulate
gtkwave traces/synthetic/checkpoint2.vcd
```

Key signals: `event_valid`, `operation`, `page_id`, `resp_hit`, `resp_miss`,
`resp_entry_id`, `resp_meta_*`, `hw_timestamp`.

## Layout

```text
rtl/kv_metadata_engine.sv   NUM_PAGES-entry table + lookup
rust/src/trace_gen.rs       workload generator
rust/src/metrics.rs         CSV metrics
rust/src/model.rs           MetadataTable reference model
docs/protocol.md            pipeline + hit/miss semantics
```

## Documentation

- [docs/protocol.md](docs/protocol.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/decisions.md](docs/decisions.md)
