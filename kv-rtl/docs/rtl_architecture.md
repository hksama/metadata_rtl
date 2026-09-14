# RTL (SystemVerilog) Architecture

This document describes the synthesizable hardware design: modules, data flow,
parameters, and what the RTL implements through Checkpoint 3.

For Rust simulation/evaluation see [rust_architecture.md](rust_architecture.md).  
For the Rust vs hardware split see [implementation_map.md](implementation_map.md).

---

## Design goal

Implement a **KV page metadata table** with **hotness scoring** in hardware:

- Track up to `NUM_PAGES` KV pages
- Accept allocate / access / release events
- Report hit, miss, or error
- Compute an integer **hotness score** per page using a compile-time selectable policy

The RTL does **not** know about prompts, tokens, or natural language — only numeric
`page_id`, `request_id`, timestamps, and opcodes.

---

## Module hierarchy

```text
top.sv
  └── kv_metadata_engine.sv
        ├── kv_hotness_engine.sv  (× NUM_PAGES, one per table slot)
        └── uses kv_pkg.sv functions
```

| File | Module | Role |
|------|--------|------|
| `rtl/kv_pkg.sv` | `kv_pkg` (package) | Widths, opcodes, policy IDs, shared functions |
| `rtl/kv_hotness_engine.sv` | `kv_hotness_engine` | Per-entry combinational hotness scorer |
| `rtl/kv_metadata_engine.sv` | `kv_metadata_engine` | Metadata table + lookup + updates |
| `rtl/top.sv` | `top` | Top-level wrapper, passes parameters |

**Detailed per-file docs:** [rtl/README.md](rtl/README.md) — ports, pipeline, op handlers, policy functions.

---

## Data flow (Checkpoint 3)

```text
                    event_valid, page_id, operation, …
                              │
                              ▼
                    ┌───────────────────┐
                    │ kv_metadata_engine │
                    │  • parallel lookup │
                    │  • table update    │
                    │  • 2-cycle pipeline│
                    └─────────┬─────────┘
                              │
              per-entry       │
                              ▼
                    ┌───────────────────┐
                    │ kv_hotness_engine  │  (× NUM_PAGES)
                    │  policy_update     │
                    │  policy_score      │
                    └─────────┬─────────┘
                              │
                              ▼
                    resp_hotness_score
                    table_hotness_score[*]
```

**Policy logic is logically separated** from metadata storage (`design_principles.md`):
- Table stores raw fields (`access_count`, `last_access`, `aging_counter`, …)
- Hotness engine computes `hotness_score` from those fields

Shared policy functions live in `kv_pkg.sv` so the metadata engine and hotness
modules use identical math.

---

## Parameters (compile-time)

Set via Makefile → `scripts/verilate.sh` → Verilator `-G` flags:

| Parameter | Default | Makefile variable |
|-----------|---------|-------------------|
| `NUM_PAGES` | 16 | `NUM_PAGES=64` |
| `POLICY` | 0 (FREQUENCY) | `POLICY=FREQUENCY\|AGING2\|AGING4\|LRU` |
| `SCORE_WIDTH` | 8 | `SCORE_WIDTH=8` |
| `AGING_INTERVAL` | 64 | `AGING_INTERVAL=256` |

Fixed widths in `kv_pkg.sv` (not yet overridable at build):

| Parameter | Default |
|-----------|---------|
| `PAGE_ID_WIDTH` | 16 |
| `REQUEST_ID_WIDTH` | 16 |
| `COUNTER_WIDTH` | 16 |
| `TIMESTAMP_WIDTH` | 16 |
| `REFCOUNT_WIDTH` | 8 |
| `STATE_WIDTH` | 2 |
| `AGING_COUNTER_MAX_WIDTH` | 4 |

---

## `kv_pkg.sv` — shared definitions

### Opcodes

| Value | Name | Effect |
|-------|------|--------|
| 0 | `OP_READ` | Access (hit/miss) |
| 1 | `OP_WRITE` | Access (hit/miss) |
| 2 | `OP_RELEASE` | Decrement refcount |
| 3 | `OP_ALLOCATE` | Claim free slot |
| 4 | `OP_ACCESS` | Access (hit/miss) |

READ, WRITE, and ACCESS share the same metadata update path.

### Policy IDs

| ID | Name | Score formula |
|----|------|---------------|
| 0 | `POLICY_FREQUENCY` | `access_count` (truncated to `SCORE_WIDTH`) |
| 1 | `POLICY_AGING_2BIT` | `aging_counter[1:0]` after update/aging |
| 2 | `POLICY_AGING_4BIT` | `aging_counter[3:0]` after update/aging |
| 3 | `POLICY_APPROX_LRU` | `MAX_TS - (hw_timestamp - last_access)` |

### Key functions

| Function | Role |
|----------|------|
| `sat_inc_u` / `sat_inc_ref` / `sat_dec_ref` | Saturating counter arithmetic |
| `policy_update_aging()` | On access: increment aging counter; on tick: `>> 1` |
| `policy_score()` | Integer hotness from metadata (no floating point) |
| `is_access_op()` | True for READ, WRITE, ACCESS |

---

## `kv_metadata_engine.sv` — metadata table

### Per-entry storage

| Field | Width | Notes |
|-------|-------|-------|
| `entry_valid` | 1 | Slot occupied |
| `entry_page_id` | PAGE_ID_WIDTH | Lookup key |
| `entry_request_id` | REQUEST_ID_WIDTH | Owner request |
| `entry_access_count` | COUNTER_WIDTH | Saturating |
| `entry_last_access` | TIMESTAMP_WIDTH | From event timestamp |
| `entry_refcount` | REFCOUNT_WIDTH | Saturating |
| `entry_state` | STATE_WIDTH | INVALID / VALID |
| `entry_aging_counter` | AGING_COUNTER_MAX_WIDTH | Used by aging policies |

### Lookup

- **Parallel compare**: all `NUM_PAGES` entries checked against latched `ev_page_id`
- Last matching index wins (duplicate keys should not occur)
- Free slot: lowest index where `!valid`

See [rtl/kv_metadata_engine.md](rtl/kv_metadata_engine.md) for commit-cycle op handlers and aging paths.

### Pipeline (2 cycles)

```text
Cycle N:   event_valid=1  → latch event internally (ev_pending)
Cycle N+1: commit         → update table, pulse resp_valid, resp_hit/miss/error
```

`hw_timestamp` increments every cycle (including idle).

### Global aging

For AGING2/AGING4 policies, when:

```text
hw_timestamp != 0  AND  hw_timestamp % AGING_INTERVAL == 0
```

all valid entries' `aging_counter` are right-shifted by 1 (on idle or before commit).

### Simulation-only port

| Signal | Role |
|--------|------|
| `table_read`, `table_idx` | Combinational read of slot `table_idx` |
| `table_*` | Metadata + `table_hotness_score` for that slot |

Used by C++ simulator for differential testing; not required in silicon.

---

## `kv_hotness_engine.sv` — per-entry scorer

One instance per table slot (generate loop in metadata engine).

### Interface

| Port | Direction | Role |
|------|-----------|------|
| `entry_valid` | in | Slot occupied |
| `access_count`, `last_access`, `aging_counter_in` | in | Metadata snapshot |
| `is_access` | in | This slot is being accessed this cycle |
| `do_age` | in | Global aging applies to this slot |
| `current_timestamp` | in | `hw_timestamp` |
| `aging_counter_out` | out | Updated aging counter |
| `hotness_score` | out | Combinational score |

Uses `kv_pkg::policy_update_aging` and `policy_score` internally.

---

## `top.sv` — wrapper

Re-exports all engine ports. Parameters forwarded:

```systemverilog
top #(
    .NUM_PAGES(16),
    .POLICY(0),
    .SCORE_WIDTH(8),
    .AGING_INTERVAL(64)
)
```

This is the Verilator top module (`--top-module top`).

---

## Response interface

| Output | Meaning |
|--------|---------|
| `resp_valid` | Event completed (one-cycle pulse) |
| `resp_error` | Protocol/allocation error |
| `resp_hit` | Page found and updated |
| `resp_miss` | Access to untracked page (not an error) |
| `resp_entry_id` | Table index affected |
| `resp_meta_*` | Post-commit metadata snapshot |
| `resp_hotness_score` | Score after this event |
| `hw_timestamp` | Global cycle counter |

See [protocol.md](protocol.md) for hit/miss/error truth table.

---

## Simulation stack (not RTL, but connected)

```text
Verilator Vtop
      ▲
sim/simulator.cpp   ← clock, reset, event drive, table peek, VCD dump
      ▲
sim/simulator.h     ← C ABI
      ▲
rust/verilator.rs   ← differential tests
```

RTL is verified by comparing every cycle's table state and responses against
`MetadataTable` in Rust (`model.rs`).

---

## Build

```bash
cd kv-rtl
make verilate                    # default: FREQUENCY, NUM_PAGES=16
POLICY=AGING2 make verilate
NUM_PAGES=64 SCORE_WIDTH=12 make verilate
```

Outputs under `build/verilator/` (generated, not committed).

---

## What RTL does **not** implement (yet)

| Feature | Planned checkpoint |
|---------|-------------------|
| Eviction / victim selection | CP4 |
| Cache hit rate simulation | CP4+ |
| Batched / async event processing | CP5 |
| Runtime software integration | CP6 |
| Real LLM traces | CP7 |
| NUMA / tiered placement | CP8 |
| Synthesis / area / timing | CP10 |
| Offline oracle / Spearman metrics | Rust only — see [implementation_map.md](implementation_map.md) |
| Runtime policy switching | Future (compile-time only today) |
| Floating-point scores | Never (integer only by design) |

---

## Related docs

| Document | Contents |
|----------|----------|
| [rtl/README.md](rtl/README.md) | Index of detailed SV file docs |
| [rtl/kv_pkg.md](rtl/kv_pkg.md) | Opcodes, policy functions, widths |
| [rtl/kv_hotness_engine.md](rtl/kv_hotness_engine.md) | Per-entry scorer |
| [rtl/kv_metadata_engine.md](rtl/kv_metadata_engine.md) | Table, pipeline, lookup, aging |
| [rtl/top.md](rtl/top.md) | Verilator top wrapper |
| [rust_architecture.md](rust_architecture.md) | Rust modules and test harness |
| [implementation_map.md](implementation_map.md) | Feature-by-feature Rust vs RTL table |
| [protocol.md](protocol.md) | Timing and response semantics |
| [hotness_policies.md](hotness_policies.md) | Policy details and experiments |
| [decisions.md](decisions.md) | Design decision log (CP2) |
