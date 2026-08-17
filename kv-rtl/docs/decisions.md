# Design decisions — Checkpoint 1

## D1.1 — Single page only

Checkpoint 1 tracks one page. `page_id` in events is checked against the
stored id; mismatches are rejected. Multi-page tables come in checkpoint 2.

## D1.2 — Two-cycle event pipeline

Events are latched on cycle *N* and committed on *N+1* so timing is explicit
and testable, consistent with checkpoint 0 infrastructure patterns.

## D1.3 — WRITE mirrors READ for metadata

`WRITE` increments `access_count` and updates `last_access`, same as `READ`.
Future checkpoints may differentiate write-side policy without changing the
event opcode layout.

## D1.4 — Saturating arithmetic

`access_count` and `refcount` use saturating increment/decrement. Overflow and
underflow never wrap silently.

## D1.5 — Rust `PageTracker` is golden

All differential tests compare Verilator output against `PageTracker`. Mismatches
print event index, event, expected metadata, and actual metadata.

## D1.6 — `hw_timestamp` wraps

The free-running hardware timestamp wraps at `2^TIMESTAMP_WIDTH`. Tests verify
wrap in the reference model; RTL and Rust stay cycle-aligned on idle ticks.

## D1.7 — Rejected events do not mutate metadata

`resp_error=1` means the latched event had no effect on stored metadata.
