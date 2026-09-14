# `kv_metadata_engine.sv` — Metadata Table Engine

**Path:** `rtl/kv_metadata_engine.sv`  
**Module:** `kv_metadata_engine`

The core of the design: a multi-page metadata table with parallel lookup, a 2-cycle
event pipeline, global aging, and integrated hotness scoring.

---

## Parameters

| Parameter | Default | Verilator flag |
|-----------|---------|----------------|
| `NUM_PAGES` | 16 | `-GNUM_PAGES=` |
| `ENTRY_IDX_WIDTH` | `$clog2(NUM_PAGES)` | (derived) |
| `POLICY` | 0 (FREQUENCY) | `-GPOLICY=` |
| `SCORE_WIDTH` | 8 | `-GSCORE_WIDTH=` |
| `AGING_INTERVAL` | 64 | `-GAGING_INTERVAL=` |

---

## Port list

### Clock and reset

| Port | Dir | Description |
|------|-----|-------------|
| `clk` | in | Rising-edge clock |
| `rst` | in | Synchronous reset (active high) |

### Event input (cycle N — accept)

| Port | Width | Description |
|------|-------|-------------|
| `event_valid` | 1 | New event presented |
| `request_id` | 16 | Request identifier |
| `page_id` | 16 | Page lookup key |
| `event_timestamp` | 16 | Timestamp for access ops |
| `operation` | 3 | Opcode (`OP_*`) |

### Event response (cycle N+1 — commit)

| Port | Width | Description |
|------|-------|-------------|
| `resp_valid` | 1 | One-cycle pulse: event completed |
| `resp_error` | 1 | Protocol/allocation error |
| `resp_hit` | 1 | Page found and updated |
| `resp_miss` | 1 | Access to untracked page |
| `resp_entry_id` | `ENTRY_IDX_WIDTH` | Table index affected |
| `resp_meta_valid` | 1 | Metadata snapshot valid |
| `resp_meta_page_id` | 16 | |
| `resp_meta_request_id` | 16 | |
| `resp_meta_access_count` | 16 | |
| `resp_meta_last_access` | 16 | |
| `resp_meta_refcount` | 8 | |
| `resp_meta_state` | 2 | |
| `resp_meta_aging_counter` | 4 | |
| `resp_hotness_score` | `SCORE_WIDTH` | Score after this event |

### Table debug port (simulation)

| Port | Width | Description |
|------|-------|-------------|
| `table_read` | 1 | Qualify indexed read |
| `table_idx` | `ENTRY_IDX_WIDTH` | Slot index `[0, NUM_PAGES)` |
| `table_valid` … `table_hotness_score` | | Combinational read of slot |

### Global

| Port | Width | Description |
|------|-------|-------------|
| `hw_timestamp` | 16 | Free-running cycle counter (+1 every clock) |

---

## Internal storage

### Per-entry registers (arrays `[0 .. NUM_PAGES-1]`)

| Array | Width | Reset value |
|-------|-------|-------------|
| `entry_valid` | 1 | 0 |
| `entry_page_id` | 16 | 0 |
| `entry_request_id` | 16 | 0 |
| `entry_access_count` | 16 | 0 |
| `entry_last_access` | 16 | 0 |
| `entry_refcount` | 8 | 0 |
| `entry_state` | 2 | INVALID |
| `entry_aging_counter` | 4 | 0 |

### Pipeline latch (single pending event)

| Register | Description |
|----------|-------------|
| `ev_pending` | Event latched, awaiting commit |
| `ev_request_id`, `ev_page_id`, `ev_timestamp`, `ev_operation` | Latched event fields |

When `ev_pending=1`, no new `event_valid` is accepted.

---

## 2-cycle pipeline

```text
Cycle N   (accept):
  event_valid=1  →  latch into ev_* , ev_pending=1
  resp_* cleared to idle defaults

Cycle N+1 (commit):
  ev_pending=1   →  compute next_* from latched event
                    pulse resp_valid
                    update entry_* arrays
                    ev_pending=0

Cycle N+2 (idle):
  event_valid=0  →  no response (unless global aging on idle)
```

**Throughput:** one event every 2 cycles minimum (accept/commit alternating).

### Sequential block summary (`always_ff`)

```text
if (rst)           → clear everything
else
  hw_timestamp++   → every cycle

  if (ev_pending)   → commit path (resp + table update)
  else if (event_valid) → latch event
  else if (global_age_tick) → age all valid entries in place
```

Response flags (`resp_hit`, `resp_miss`, `resp_error`) are pulsed only on commit;
default to 0 on non-commit cycles.

---

## Lookup logic

### Page lookup (`always_comb`)

Scans all `NUM_PAGES` entries in parallel:

```systemverilog
if (entry_valid[i] && entry_page_id[i] == ev_page_id)
    lk_found = 1; lk_entry_id = i;
```

If multiple matches existed, last index wins (should not happen with correct use).

Uses **latched** `ev_page_id` when `ev_pending` (not live input bus).

### Free slot search (`always_comb`)

Lowest index where `!entry_valid`:

```systemverilog
if (!entry_valid[i] && !free_found)
    free_found = 1; free_entry_id = i;
```

---

## Operation behavior (commit cycle)

### `OP_ALLOCATE`

| Condition | Result |
|-----------|--------|
| Page already tracked (`lk_found`) | `resp_error=1` |
| Table full (`!free_found`) | `resp_error=1` |
| Success | New entry at `free_entry_id`, `refcount=1`, counters zero |

Does not set `resp_hit` or `resp_miss`.

### `OP_READ` / `OP_WRITE` / `OP_ACCESS`

| Condition | Result |
|-----------|--------|
| Page not tracked | `resp_miss=1`, table unchanged |
| Hit | `resp_hit=1`, increment count, update `last_access`, aging, score |

On hit, aging and score use **post-update** values via inline `policy_*` calls.

### `OP_RELEASE`

| Condition | Result |
|-----------|--------|
| Page not tracked | `resp_error=1` |
| `refcount==0` | `resp_error=1` |
| Success | `resp_hit=1`, `refcount--` (saturate at 0) |

Hotness score unchanged (read from `entry_hotness_score[lk_entry_id]`).

### Unknown opcode

`resp_error=1`.

---

## Global aging

### Tick condition

```systemverilog
global_age_tick =
    (AGING_INTERVAL != 0) &&
    (hw_timestamp != 0) &&
    (hw_timestamp % AGING_INTERVAL == 0) &&
    (POLICY is AGING_2BIT or AGING_4BIT);
```

Only active for aging policies. Frequency and LRU ignore the interval.

### Two aging paths

1. **On commit cycle** (`ev_pending` + `global_age_tick`): in combinational `next_*` logic,
   all valid entries except the accessed entry get `aging_counter >> 1`.

2. **On idle cycle** (no pending event): in sequential block, all valid entries
   get `entry_aging_counter >>= 1` registered update.

The accessed entry on an access commit gets `policy_update_aging(..., is_access=1)` instead.

---

## Hotness integration

### Generate loop `gen_hotness`

Instantiates `NUM_PAGES` copies of `kv_hotness_engine`, producing
`entry_hotness_score[0..NUM_PAGES-1]` continuously from registered metadata.

### Response score selection

| Operation | `resp_hotness_score` source |
|-----------|----------------------------|
| ALLOCATE | `entry_hotness_score[free_entry_id]` (usually 0) |
| ACCESS hit | Inline `policy_score` on updated metadata |
| RELEASE | `entry_hotness_score[lk_entry_id]` (unchanged metadata) |
| MISS / ERROR | 0 (defaults) |

---

## Table read port

When `table_read=1`:

```systemverilog
table_* = entry_*[table_idx]   // registered metadata
table_hotness_score = entry_hotness_score[table_idx]  // combinational score
```

Purely combinational. Used by C simulator for differential testing; not required
in a production integration.

---

## Reset behavior

On `rst`:

- Clears `ev_pending` and all response registers
- Clears all `entry_*` arrays (`entry_valid=0`)
- `hw_timestamp=0`

Two reset cycles applied by testbench before operation.

---

## Area / timing considerations

| Aspect | Current implementation |
|--------|------------------------|
| Lookup | O(`NUM_PAGES`) parallel comparators |
| Hotness | O(`NUM_PAGES`) `kv_hotness_engine` instances |
| Table update | Full array copy via `next_*` → `entry_*` on commit |
| Pipeline | Single in-flight event |

Indexed/direct-mapped lookup deferred to future checkpoints.

---

## Related docs

- [kv_pkg.md](kv_pkg.md) — policy functions and opcodes
- [kv_hotness_engine.md](kv_hotness_engine.md) — per-entry scorer
- [../protocol.md](../protocol.md) — hit/miss/error truth table
