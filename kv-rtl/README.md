# KV-RTL — Checkpoint 0

Infrastructure for KV-cache metadata RTL development: SystemVerilog, Verilator,
Rust harness, and waveform debug.

This checkpoint validates the build/simulation pipeline only. `rtl/top.sv` is a
placeholder (`output = input + 1`, one-cycle latency) — not KV-cache logic.

## Prerequisites

- [Verilator](https://www.veripool.org/verilator/) 5.x
- Rust toolchain (`cargo`)
- `g++`, `make`
- [GTKWave](https://gtkwave.sourceforge.net/) (optional, for waveforms)

## Quick start

```bash
cd kv-rtl
make test
```

`make test` will:

1. Verilate `rtl/top.sv`
2. Build the C++ cycle-by-cycle testbench
3. Run C++ and Rust validation
4. Exit non-zero on any failure

## Other commands

| Command          | Description                                      |
|------------------|--------------------------------------------------|
| `make verilate`  | Build Verilator model and C++ test binary        |
| `make simulate`  | Run tests and capture a waveform under `traces/` |
| `make clean`     | Remove build artifacts                           |

## Waveforms

Generate a VCD trace:

```bash
make simulate
gtkwave traces/synthetic/checkpoint0.vcd
```

FST output:

```bash
WAVE_FMT=fst make simulate
gtkwave traces/synthetic/checkpoint0.fst
```

Inspect `clk`, `rst`, `input_valid`, `input_data`, `output_valid`, and
`output_data` under the `top` scope.

See [traces/README.md](traces/README.md) for details.

## Layout

```text
rtl/        SystemVerilog sources
sim/        Verilator C++ wrapper (C ABI)
tests/      C++ cycle tests
rust/       Rust harness and Simulator trait
scripts/    verilate, simulate, clean helpers
docs/       architecture, protocol, decisions
traces/     waveform output
```

## Rust harness

The Rust crate defines a `Simulator` trait; `VerilatorSimulator` is the only
backend today. Tests check reset behavior, one-cycle latency, arithmetic, and
protocol violations.

```bash
cd rust
cargo test --release
cargo run --release
```

## Documentation

- [docs/architecture.md](docs/architecture.md)
- [docs/protocol.md](docs/protocol.md)
- [docs/decisions.md](docs/decisions.md)
- [KV_RTL_MVP_ROADMAP.md](KV_RTL_MVP_ROADMAP.md)
