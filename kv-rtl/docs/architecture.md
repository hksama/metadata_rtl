# Checkpoint 1 — Architecture

## Components

```text
Rust harness + PageTracker reference model
        │
        ▼
MetadataSimulator trait
        │
        ▼
C ABI (sim/simulator.h)
        │
        ▼
Verilator (top → kv_page_tracker)
        │
        ▼
rtl/kv_pkg.sv + rtl/kv_page_tracker.sv
```

## RTL modules

| Module | Role |
|--------|------|
| `kv_pkg` | Width parameters, opcodes, saturation helpers |
| `kv_page_tracker` | Single-page metadata storage and event processing |
| `top` | Thin wrapper exposing the tracker as the simulation top |

## Data tracked per page

- `page_id`, `request_id`
- `access_count` (saturating)
- `last_access` (from event timestamp on READ/WRITE)
- `refcount` (saturating increment on ALLOCATE, decrement on RELEASE)
- `valid` / `state`

Policy (LRU, frequency, etc.) is **not** implemented here — only metadata
storage and explicit updates from access events (Principle 2).

## Timing model

Two-cycle event pipeline: latch on cycle *N*, commit on cycle *N+1*.
`hw_timestamp` increments every committed cycle.

See [protocol.md](protocol.md) for signal-level detail.
