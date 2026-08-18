# Checkpoint 3 — Hotness Policy Architecture

## Data flow

```text
ACCESS EVENT
      │
      ▼
┌─────────────────┐
│ Metadata Table  │  page_id, request_id, refcount,
│                 │  access_count, last_access, aging_counter
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Hotness Engine  │  policy_update() / policy_score()
│ (kv_hotness_    │
│  engine.sv)     │
└────────┬────────┘
         │
         ▼
   HOTNESS SCORE (integer, SCORE_WIDTH bits)
```

The hotness engine is **logically separated** from metadata storage so policies
can be swapped independently.

## RTL modules

| Module | Role |
|--------|------|
| `kv_pkg` | Widths, policy IDs, `policy_update_aging()`, `policy_score()` |
| `kv_hotness_engine` | Per-entry hotness scoring module |
| `kv_metadata_engine` | Metadata table + hotness integration |
| `top` | Parameterized wrapper |

## Policies (compile-time selection)

| Policy | Score | Metadata extra |
|--------|-------|----------------|
| `POLICY_FREQUENCY` | `access_count` | — |
| `POLICY_AGING_2BIT` | `aging_counter[1:0]` | 2-bit counter, global aging |
| `POLICY_AGING_4BIT` | `aging_counter[3:0]` | 4-bit counter, global aging |
| `POLICY_APPROX_LRU` | `MAX_TS - (now - last_access)` | uses `last_access` |

```bash
make POLICY=FREQUENCY
make POLICY=AGING2 AGING_INTERVAL=64
make POLICY=AGING4 AGING_INTERVAL=256
make POLICY=LRU
```

## Score representation

Integer/fixed-point only (`SCORE_WIDTH` default 8). No floating point —
unsuitable for synthesizable RTL (area, latency, non-deterministic rounding).

## Rust evaluation (offline)

| Module | Role |
|--------|------|
| `policy.rs` | Software mirror of RTL policies + hardware cost estimates |
| `oracle.rs` | Ground-truth future reuse oracle (not in RTL) |
| `eval.rs` | Classification + ranking metrics vs oracle |
| `trace_gen.rs` | Sequential, random, hot, bursty, Zipf, agentic workloads |

Run policy evaluation:

```bash
make policy-test
make aging-sweep
```
