# `top.sv` — Top-Level Wrapper

**Path:** `rtl/top.sv`  
**Module:** `top`  
**Verilator top:** `--top-module top`

Thin wrapper that exposes all `kv_metadata_engine` ports at the chip/sim boundary and
forwards compile-time parameters. No additional logic.

---

## Parameters

Passed through to `u_engine`:

| Parameter | Default | Makefile / Verilator |
|-----------|---------|----------------------|
| `NUM_PAGES` | 16 | `NUM_PAGES=64 make verilate` |
| `POLICY` | 0 (FREQUENCY) | `POLICY=AGING2 make verilate` |
| `SCORE_WIDTH` | 8 | `SCORE_WIDTH=12 make verilate` |
| `AGING_INTERVAL` | 64 | `AGING_INTERVAL=256 make verilate` |

---

## Module instantiation

```systemverilog
kv_metadata_engine #(
    .NUM_PAGES(NUM_PAGES),
    .POLICY(POLICY),
    .SCORE_WIDTH(SCORE_WIDTH),
    .AGING_INTERVAL(AGING_INTERVAL)
) u_engine ( ... );
```

All ports are direct passthrough — no renaming or gating.

---

## Port map

Ports match `kv_metadata_engine` exactly. Width of `resp_entry_id` and `table_idx`
uses:

```systemverilog
kv_pkg::entry_idx_width(NUM_PAGES)
```

So for `NUM_PAGES=16` → 4 bits; for `NUM_PAGES=64` → 6 bits.

---

## Build integration

`scripts/verilate.sh` compiles:

```text
rtl/kv_pkg.sv
rtl/kv_hotness_engine.sv
rtl/kv_metadata_engine.sv
rtl/top.sv
```

With flags:

```bash
-GNUM_PAGES="${NUM_PAGES}"
-GPOLICY="${POLICY_ID}"
-GSCORE_WIDTH="${SCORE_WIDTH}"
-GAGING_INTERVAL="${AGING_INTERVAL}"
```

Policy name mapping in shell script:

| `POLICY` env | `POLICY_ID` |
|--------------|-------------|
| FREQUENCY, FREQ | 0 |
| AGING2, AGING_2BIT | 1 |
| AGING4, AGING_4BIT | 2 |
| LRU, APPROX_LRU | 3 |

---

## Simulation connection

```text
top (Verilator Vtop)
  ↑ driven by sim/simulator.cpp
  ↑ read by rust/verilator.rs via C ABI
```

C++ simulator connects:

- `clk`, `rst` — clock and reset
- Event inputs — from `sim_drive_event()`
- Response outputs — read by `sim_resp_*()` and `sim_read_resp_metadata()`
- `table_read` / `table_idx` — from `sim_read_table_entry()`

---

## Synthesis note

For ASIC/FPGA integration, `top` is the entity to constrain. The `table_read` debug
port can be tied off or removed in production builds.

See [kv_metadata_engine.md](kv_metadata_engine.md) for internal behavior.
