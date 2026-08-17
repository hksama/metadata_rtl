# Checkpoint 1 — Single-page metadata protocol

## Scope

One KV page is tracked. The engine accepts **access events** and updates
metadata with a fixed **two-cycle** pipeline (accept, then commit).

Configurable widths are defined in `rtl/kv_pkg.sv`:

| Parameter | Default |
|-----------|---------|
| `PAGE_ID_WIDTH` | 16 |
| `REQUEST_ID_WIDTH` | 16 |
| `COUNTER_WIDTH` | 16 |
| `TIMESTAMP_WIDTH` | 16 |
| `REFCOUNT_WIDTH` | 8 |
| `OP_WIDTH` | 3 |

## Event interface (`ACCESS_EVENT`)

| Signal | Width | Description |
|--------|-------|-------------|
| `event_valid` | 1 | Qualifies the event for the current cycle |
| `request_id` | `REQUEST_ID_WIDTH` | Owning request |
| `page_id` | `PAGE_ID_WIDTH` | Target page |
| `event_timestamp` | `TIMESTAMP_WIDTH` | Software-supplied timestamp for access recording |
| `operation` | `OP_WIDTH` | Event opcode (see below) |

### Operations

| Opcode | Name | Behavior |
|--------|------|----------|
| `0` | `READ` | `access_count += 1` (saturate), `last_access = event_timestamp` |
| `1` | `WRITE` | Same metadata effect as `READ` in checkpoint 1 |
| `2` | `RELEASE` | `refcount -= 1` (saturate at 0); error if already 0 |
| `3` | `ALLOCATE` | `valid=1`, store ids, `refcount=1`, counters cleared |

Reserved opcodes are rejected with `resp_error=1`.

## Metadata outputs

| Signal | Description |
|--------|-------------|
| `meta_valid` | Page is allocated |
| `meta_page_id` | Stored page id |
| `meta_request_id` | Stored request id |
| `meta_access_count` | Saturating access counter |
| `meta_last_access` | Last recorded `event_timestamp` from READ/WRITE |
| `meta_refcount` | Saturating reference count |
| `meta_state` | `0=INVALID`, `1=VALID` |

## Response interface

| Signal | Description |
|--------|-------------|
| `resp_valid` | High for one cycle when an event completes |
| `resp_error` | High if the event was rejected |

Rejected events do **not** modify metadata.

## Cycle-level timing

The tracker accepts at most **one event every two cycles**.

### Cycle N — accept (latch)

- `event_valid = 1` with stable `request_id`, `page_id`, `event_timestamp`, `operation`
- `hw_timestamp` increments on the posedge
- Event is latched internally; `resp_valid = 0`

### Cycle N+1 — commit

- `event_valid = 0` (recommended)
- `hw_timestamp` increments on the posedge
- Latched event is applied
- `resp_valid = 1` for this cycle
- `resp_error` indicates success (`0`) or rejection (`1`)

### Example

```text
Cycle 10: event_valid=1, page_id=7, operation=READ
Cycle 11: metadata update committed, resp_valid=1
```

## Hardware timestamp (`hw_timestamp`)

- Increments by 1 every posedge while `rst=0`
- Holds at 0 while `rst=1`
- Wraps modulo `2^TIMESTAMP_WIDTH` (`65535 → 0` with default widths)
- Independent from `event_timestamp` (software supplied)

## Saturation and error rules

| Condition | Result |
|-----------|--------|
| `access_count == MAX` and READ/WRITE | Stay at `MAX` |
| `refcount == MAX` and ALLOCATE | N/A (duplicate alloc rejected first) |
| `refcount == 0` and RELEASE | `resp_error=1`, no change |
| `refcount > 0` and RELEASE | `refcount -= 1` (saturate at 0) |
| READ/WRITE/RELEASE when `meta_valid=0` | `resp_error=1` |
| READ/WRITE/RELEASE with mismatched `page_id` | `resp_error=1` |
| ALLOCATE when `meta_valid=1` | `resp_error=1` (duplicate allocation) |
| Unknown opcode | `resp_error=1` |

## Reset

Synchronous active-high `rst`:

- Clears metadata to invalid/zeroed state
- Clears pending latched events
- Clears `resp_valid` / `resp_error`
- Clears `hw_timestamp`
- Events presented during reset are ignored

## Differential testing

The Rust `PageTracker` reference model mirrors this protocol. Tests feed
identical event streams to Rust and Verilator and compare metadata,
`hw_timestamp`, and response flags after every event.
