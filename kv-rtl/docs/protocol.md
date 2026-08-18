# Checkpoint 2 — Multi-page metadata protocol

## Scope

`NUM_PAGES` metadata slots (default **16**, compile-time via `-GNUM_PAGES=`).
Each slot stores: `page_id`, `request_id`, `access_count`, `last_access`,
`refcount`, `valid`, `state`.

Lookup is **parallel compare** across all entries (correctness-first, not optimized).

## Event interface

Same as checkpoint 1, plus `OP_ACCESS` (opcode 4) as a unified access operation.
`READ`, `WRITE`, and `ACCESS` all perform the same metadata update.

| Opcode | Name | Behavior |
|--------|------|----------|
| 0 | READ | Access (hit/miss) |
| 1 | WRITE | Access (hit/miss) |
| 2 | RELEASE | Decrement `refcount` on tracked page |
| 3 | ALLOCATE | Claim a free slot for `page_id` |
| 4 | ACCESS | Access (hit/miss) |

## Response interface

| Signal | Description |
|--------|-------------|
| `resp_valid` | Event completed (one-cycle pulse) |
| `resp_error` | Protocol/allocation error |
| `resp_hit` | Page found and updated |
| `resp_miss` | Access to untracked `page_id` (not an error) |
| `resp_entry_id` | Table index of affected entry |
| `resp_meta_*` | Metadata of affected entry after commit |

### HIT vs MISS vs ERROR

| Outcome | `resp_hit` | `resp_miss` | `resp_error` | Table change |
|---------|------------|-------------|--------------|--------------|
| Access hit | 1 | 0 | 0 | Yes |
| Access miss | 0 | 1 | 0 | No |
| Alloc success | 0 | 0 | 0 | Yes |
| Alloc fail (full/duplicate) | 0 | 0 | 1 | No |
| Release success | 1 | 0 | 0 | Yes |
| Release fail | 0 | 0 | 1 | No |

A **miss is not an eviction** — the engine only reports “page not tracked.”

## Pipeline (critical path)

```text
event_valid (input)
      ↓  cycle N — latch
parallel page_id lookup (combinational, all entries)
      ↓  cycle N+1 — commit
metadata read + update
      ↓
resp_valid / resp_hit / resp_miss / resp_meta_* (output)
```

### Latency and throughput (functional cycles)

| Metric | Value |
|--------|-------|
| Event accept → response | **2 cycles** |
| Throughput | **1 event / 2 cycles** (back-to-back with `event_valid` every other cycle) |
| Idle `hw_timestamp` | +1 per cycle |

Interface accepts **one event per commit**. Multi-event per cycle is reserved for
future extension via a wider event bus.

## Table read port (simulation)

| Signal | Description |
|--------|-------------|
| `table_read` | Qualify indexed read |
| `table_idx` | Entry index `[0, NUM_PAGES)` |
| `table_*` | Combinational read of slot metadata |

## ALLOCATE rules

1. Reject if `page_id` already tracked (`resp_error`).
2. Find lowest-index free slot.
3. Reject if table full (`resp_error`).
4. On success: initialize entry, `refcount=1`, return `resp_entry_id`.

## RELEASE rules

1. Reject if page not tracked or `refcount==0`.
2. On success: `refcount -= 1` (saturate at 0).

## Saturation

`access_count` and `refcount` use saturating increment/decrement (unchanged from CP1).

## Rust reference model

`MetadataTable` mirrors this protocol. Differential tests compare full table state,
response flags, and `resp_metadata` after every event.

## Workload artifacts

- `traces/synthetic/checkpoint2_trace.txt` — human-readable event listing (`A`/`C`/`R`)
- `traces/synthetic/checkpoint2_metrics.csv` — events, cycles, hits, misses, etc.
