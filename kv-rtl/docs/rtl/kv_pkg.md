# `kv_pkg.sv` — Shared Package

**Path:** `rtl/kv_pkg.sv`  
**Type:** SystemVerilog package (no ports; imported by all modules)

The package centralizes bit widths, opcode encodings, policy identifiers, and
pure functions used by both the metadata table and hotness engine. Keeping policy
math here ensures a single definition shared across modules.

---

## Parameters

### Table and field widths

| Parameter | Default | Description |
|-----------|---------|-------------|
| `DEFAULT_NUM_PAGES` | 16 | Default table size (overridden on `top` / `kv_metadata_engine`) |
| `PAGE_ID_WIDTH` | 16 | Width of `page_id` |
| `REQUEST_ID_WIDTH` | 16 | Width of `request_id` |
| `COUNTER_WIDTH` | 16 | Width of `access_count` |
| `TIMESTAMP_WIDTH` | 16 | Width of `last_access`, `event_timestamp`, `hw_timestamp` |
| `REFCOUNT_WIDTH` | 8 | Width of `refcount` |
| `OP_WIDTH` | 3 | Width of `operation` opcode bus |
| `STATE_WIDTH` | 2 | Width of entry `state` field |

### Hotness policy (Checkpoint 3)

| Parameter | Default | Description |
|-----------|---------|-------------|
| `DEFAULT_SCORE_WIDTH` | 8 | Internal score width before truncation |
| `AGING_COUNTER_MAX_WIDTH` | 4 | Storage width for aging counter (2 or 4 bits used) |
| `DEFAULT_AGING_INTERVAL` | 64 | Default cycles between global aging ticks |

---

## Policy identifiers

```systemverilog
typedef int policy_e;
localparam policy_e POLICY_FREQUENCY   = 0;
localparam policy_e POLICY_AGING_2BIT  = 1;
localparam policy_e POLICY_AGING_4BIT  = 2;
localparam policy_e POLICY_APPROX_LRU  = 3;
```

Selected at Verilate time via `-GPOLICY=<0..3>` (Makefile maps names to IDs).

---

## Opcodes (`op_t`)

| Constant | Value | Category |
|----------|-------|----------|
| `OP_READ` | 0 | Access |
| `OP_WRITE` | 1 | Access |
| `OP_RELEASE` | 2 | Refcount |
| `OP_ALLOCATE` | 3 | Table management |
| `OP_ACCESS` | 4 | Access |

Access operations (`READ`, `WRITE`, `ACCESS`) share the same metadata update path.

---

## State encodings (`state_t`)

| Constant | Value |
|----------|-------|
| `STATE_INVALID` | 0 |
| `STATE_VALID` | 1 |

---

## Utility functions

### `entry_idx_width(num_pages)`

Returns `$clog2(num_pages)` (minimum 1). Used for `resp_entry_id` and `table_idx` widths.

### `aging_counter_width(policy)`

Returns effective aging counter bits: 2 for AGING_2BIT, 4 for AGING_4BIT, 0 otherwise.

### `is_access_op(op)`

Returns true if `op` is `OP_READ`, `OP_WRITE`, or `OP_ACCESS`.

---

## Saturation arithmetic

These mirror the Rust helpers in `types.rs` and must stay in sync for differential tests.

### `sat_inc_u(value)` → `COUNTER_WIDTH`

Increment `access_count`. Holds at all-ones (`COUNTER_MAX`).

### `sat_inc_ref(value)` → `REFCOUNT_WIDTH`

Increment on allocate. Holds at all-ones.

### `sat_dec_ref(value)` → `REFCOUNT_WIDTH`

Decrement on release. Holds at zero.

### `sat_inc_aging(value, max_val)` → `AGING_COUNTER_MAX_WIDTH`

Increment aging counter, saturating at `max_val` (3 for 2-bit, 15 for 4-bit).

---

## Hotness policy functions

These implement the hardware policy interface:

```text
policy_update_aging(...)  →  updated aging counter
policy_score(...)         →  integer hotness score
```

### `policy_update_aging(policy, aging_in, is_access, do_age)`

| Input | Meaning |
|-------|---------|
| `policy` | One of `POLICY_*` constants |
| `aging_in` | Current aging counter |
| `is_access` | This cycle is an access to this entry |
| `do_age` | Global aging tick applies |

| Condition | Result |
|-----------|--------|
| `do_age` | `aging_in >> 1` |
| `is_access` + AGING_2BIT | `min(aging_in + 1, 3)` |
| `is_access` + AGING_4BIT | `min(aging_in + 1, 15)` |
| Otherwise | `aging_in` unchanged |

Frequency and LRU policies do not modify the aging counter here.

### `policy_score(policy, entry_valid, access_count, last_access, aging_counter, current_timestamp)`

Returns `DEFAULT_SCORE_WIDTH` bits. Invalid entries always score 0.

| Policy | Formula |
|--------|---------|
| `POLICY_FREQUENCY` | `access_count[7:0]` (low bits of counter) |
| `POLICY_AGING_2BIT` | Zero-extend `aging_counter[1:0]` |
| `POLICY_AGING_4BIT` | Zero-extend `aging_counter[3:0]` |
| `POLICY_APPROX_LRU` | `(2^TIMESTAMP_WIDTH - 1) - (current_timestamp - last_access)` |

**No floating point.** Integer subtraction for LRU uses wrapping timestamp arithmetic.

Callers (e.g. `kv_hotness_engine`) truncate to `SCORE_WIDTH` if smaller than 8.

---

## Rust mirror

| RTL | Rust |
|-----|------|
| `sat_inc_u` | `sat_inc_u16` |
| `sat_inc_ref` / `sat_dec_ref` | `sat_inc_u8` / `sat_dec_u8` |
| `policy_update_aging` | `policy::policy_update_aging` |
| `policy_score` | `policy::policy_score` |
| Opcode constants | `types::Operation` |
