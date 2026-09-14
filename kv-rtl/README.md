# KV-RTL — Checkpoint 2

Multi-page KV metadata table with parallel lookup, hit/miss reporting,
synthetic workload generation, and metrics CSV output.

## Quick start

```bash
cd kv-rtl
make test
```

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/code_architecture.md](docs/code_architecture.md) | Documentation index |
| [docs/rust_architecture.md](docs/rust_architecture.md) | Rust modules, data flow, binaries |
| [docs/rtl_architecture.md](docs/rtl_architecture.md) | SystemVerilog modules and parameters |
| [docs/rtl/README.md](docs/rtl/README.md) | Detailed per-file SV reference |
| [docs/implementation_map.md](docs/implementation_map.md) | Rust vs RTL vs not implemented |
| [docs/protocol.md](docs/protocol.md) | Event protocol and timing |
| [docs/hotness_policies.md](docs/hotness_policies.md) | Hotness policies (CP3) |

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
