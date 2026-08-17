0.1 Repository structure

Create:

kv-rtl/
│
├── rtl/
│   ├── top.sv
│   └── ...
│
├── sim/
│   ├── ...
│
├── rust/
│   ├── ...
│
├── tests/
│   ├── ...
│
├── traces/
│   ├── synthetic/
│   └── README.md
│
├── scripts/
│   ├── verilate.sh
│   ├── simulate.sh
│   └── clean.sh
│
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   └── decisions.md
│
├── Makefile
├── README.md
└── KV_RTL_MVP_ROADMAP.md

The exact structure may be improved by the agent, but the separation between:

RTL
simulation
Rust
tests
traces
documentation

must remain.

0.2 Minimal RTL

Create:

top.sv

with:

clock
reset
input_valid
input_data
output_valid
output_data

The module should perform a trivial deterministic transformation.

Example:

input_data + 1

This is NOT part of the actual accelerator.

It is only an infrastructure test.

0.3 Verilator simulation

Verify:

reset
clock
input
output

with cycle-by-cycle checking.

The test must detect:

incorrect reset
incorrect latency
invalid output
protocol violation

0.4 Waveforms

Enable VCD/FST waveform generation.

Demonstrate that:

clock
reset
valid
input
output

can be inspected.

The repository must document how to open the waveform.

0.5 Rust simulation harness

Create a minimal Rust program that:

generates input events
invokes the Verilator model
waits for output
validates the result
prints cycle counts

The Rust side should eventually become the software reference/runtime.

Do NOT make Rust depend deeply on generated Verilator internals.

Create an abstraction layer:

Rust test/runtime
       │
       ▼
Simulator interface
       │
       ▼
Verilator

0.6 CI-style command

The project should support:

make test

which performs:

build RTL
build Verilator model
run tests
return non-zero on failure