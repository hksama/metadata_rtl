# KV-RTL — Checkpoint 1

Hardware-assisted KV-cache **single-page metadata tracker**: SystemVerilog,
Verilator, Rust reference model, and differential tests.

## Prerequisites

- Verilator 5.x
- Rust (`cargo`)
- `g++`, `make`
- GTKWave (optional)

## Quick start

```bash
cd kv-rtl
make test
```

`make test` builds RTL, runs C++ smoke tests, Rust differential tests (edge
cases + 500 random events), and exits non-zero on mismatch.

## Waveforms

```bash
make simulate
gtkwave traces/synthetic/checkpoint1.vcd
```

Signals: `event_valid`, `operation`, `page_id`, `request_id`, `event_timestamp`,
`resp_valid`, `resp_error`, `meta_*`, `hw_timestamp`.

## Layout

```text
rtl/kv_pkg.sv           parameters and opcodes
rtl/kv_page_tracker.sv  single-page metadata engine
rtl/top.sv              simulation top
rust/src/model.rs       PageTracker reference model
rust/src/types.rs       shared event/metadata types
docs/protocol.md        cycle-accurate protocol spec
```

## Documentation

- [docs/protocol.md](docs/protocol.md) — event interface and timing
- [docs/architecture.md](docs/architecture.md)
- [docs/decisions.md](docs/decisions.md)

## Checkpoint 0 note

The checkpoint 0 placeholder (`input + 1`) has been replaced by the metadata
tracker in `rtl/kv_page_tracker.sv`.
