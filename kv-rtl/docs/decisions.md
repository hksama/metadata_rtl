# Design decisions — Checkpoint 0

## D0.1 — Placeholder RTL instead of early metadata

The first module intentionally performs `input + 1` so infrastructure (build,
simulation, Rust harness, waveforms) can be validated before KV metadata logic
is added in checkpoint 1.

## D0.2 — C ABI between Rust and Verilator

Rust links against a thin C wrapper (`sim/simulator.cpp`) rather than generated
Verilator C++ classes. This keeps the Rust side stable as RTL complexity grows.

## D0.3 — One-cycle fixed latency

The placeholder pipeline registers inputs before producing outputs so tests can
detect latency regressions.

## D0.4 — Dual test entry points

Both a C++ cycle test (`tests/cpp/tb_top.cpp`) and a Rust harness exercise the
same simulator API. `make test` runs both.

## D0.5 — VCD default, FST optional

VCD is the default waveform format for broad gtkwave compatibility. Set
`WAVE_FMT=fst` or `TRACE_FMT=fst` to emit FST instead.
