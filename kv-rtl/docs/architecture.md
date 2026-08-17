# Checkpoint 0 — Architecture

This checkpoint establishes the RTL development environment only. The `top`
module is an infrastructure placeholder, not KV-cache logic.

## Layers

```text
Rust test / runtime harness
        │
        ▼
Simulator trait (kv_rtl::Simulator)
        │
        ▼
C ABI (sim/simulator.h)
        │
        ▼
Verilator model (Vtop)
        │
        ▼
RTL (rtl/top.sv)
```

## RTL behavior

`top` accepts a 32-bit word when `input_valid` is asserted. One cycle later it
asserts `output_valid` and drives `output_data = captured_input + 1`.

This exercises reset, clocking, handshake signals, and fixed latency — the
minimum needed before metadata engines are introduced in checkpoint 1.
