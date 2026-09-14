# `kv_hotness_engine.sv` — Hotness Scoring Module

**Path:** `rtl/kv_hotness_engine.sv`  
**Module:** `kv_hotness_engine`  
**Instances:** `NUM_PAGES` (one per table slot, in `gen_hotness` loop)

Logically separated from metadata storage so hotness policies can be swapped
without rewriting the table. Purely **combinational** — no registers inside this module.

---

## Purpose

For one metadata table entry, compute:

1. **Updated aging counter** after an access or global aging tick
2. **Hotness score** from current metadata snapshot

Delegates all math to `kv_pkg::policy_update_aging` and `kv_pkg::policy_score`.

---

## Parameters

| Parameter | Default | Set by |
|-----------|---------|--------|
| `POLICY` | `POLICY_FREQUENCY` (0) | Verilator `-GPOLICY=` |
| `SCORE_WIDTH` | 8 | Verilator `-GSCORE_WIDTH=` |

---

## Ports

### Metadata inputs (from table slot)

| Port | Width | Description |
|------|-------|-------------|
| `entry_valid` | 1 | Slot occupied |
| `access_count` | `COUNTER_WIDTH` | Total accesses |
| `last_access` | `TIMESTAMP_WIDTH` | Last access timestamp |
| `aging_counter_in` | `AGING_COUNTER_MAX_WIDTH` | Stored aging state |

### Control inputs

| Port | Description |
|------|-------------|
| `is_access` | Asserted when this slot is the target of a pending READ/WRITE/ACCESS |
| `do_age` | Asserted on global aging tick for this slot (and not the accessed slot) |
| `current_timestamp` | Connected to `hw_timestamp` from metadata engine |

### Outputs

| Port | Width | Description |
|------|-------|-------------|
| `aging_counter_out` | 4 | Combinational next aging value |
| `hotness_score` | `SCORE_WIDTH` | Combinational hotness score |

---

## Internal logic

```text
next_aging = policy_update_aging(
    POLICY, aging_counter_in,
    entry_valid && is_access,
    entry_valid && do_age
);

score_full = policy_score(
    POLICY, entry_valid, access_count, last_access,
    next_aging, current_timestamp
);

hotness_score = score_full[SCORE_WIDTH-1:0];
```

All logic is in one `always_comb` block.

---

## How instances are driven (in `kv_metadata_engine`)

For slot index `gi`:

```systemverilog
is_accessed_entry = ev_pending && lk_found &&
                    (lk_entry_id == gi) &&
                    is_access_op(ev_operation);

do_age_entry = global_age_tick && entry_valid[gi] && !is_accessed_entry;
```

| Signal | When true |
|--------|-----------|
| `is_access` | Pending event is an access hitting this slot |
| `do_age` | Global aging tick, slot valid, not the accessed slot |

The per-instance `aging_counter_out` is connected to `entry_aged_unused[gi]` (unused wire).
**Committed** aging updates happen in the metadata engine's sequential logic, not from
this output directly.

---

## Score vs committed state

The hotness engine continuously reflects **current registered metadata**.
On an access commit cycle, the metadata engine also calls `policy_update_aging` and
`policy_score` inline for the accessed entry to produce `resp_hotness_score` using
post-update values (`access_new_count`, `access_aged_counter`).

Per-slot `entry_hotness_score[gi]` is used for:

- `table_hotness_score` on debug read
- `resp_hotness_score` on allocate/release (combinational from current state)

---

## Synthesis notes

- No clocks or resets inside module
- Policy selected at elaboration time (`POLICY` parameter) — dead policy branches optimized away
- Integer-only arithmetic (no FP units)

See [kv_pkg.md](kv_pkg.md) for policy formulas.
