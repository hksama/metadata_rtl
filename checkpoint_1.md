Objective

Build the smallest useful piece of the actual accelerator.

Represent one KV page and maintain its metadata.

Do NOT implement multiple pages yet.

1.1 KV page abstraction

Define a conceptual page:

KV Page
│
├── page_id
├── request_id
├── access_count
├── last_access
├── refcount
├── valid
└── state

Use configurable widths.

Example parameters:

PAGE_ID_WIDTH
REQUEST_ID_WIDTH
COUNTER_WIDTH
TIMESTAMP_WIDTH
REFCOUNT_WIDTH

Do NOT use arbitrary hard-coded widths throughout the RTL.

1.2 Access event interface

Define a hardware event:

ACCESS_EVENT


valid
request_id
page_id
timestamp
operation

Initially operations may be:

READ
WRITE
RELEASE

The design should be extendable.

1.3 Metadata update behavior

For:

READ

update:

access_count += 1
last_access = timestamp

For:

RELEASE

update:

refcount -= 1

For allocation:

refcount += 1
valid = 1

All behavior must be explicitly specified.

1.4 Cycle-level specification

Write a protocol document explaining exactly when updates occur.

Example:

Cycle N:
input_valid = 1
page_id = 7


Cycle N+1:
metadata update committed

If the design has one-cycle latency, document it.

If it is zero-latency combinational logic, document that instead.

The agent must NOT leave timing semantics ambiguous.

1.5 Saturation behavior

Counters must not silently overflow.

Define behavior for:

access_count == MAX

Possible behavior:

saturate

rather than wrapping to zero.

Do the same for:

refcount

unless another behavior is justified.

1.6 Timestamp handling

Implement a monotonically increasing timestamp counter.

Consider:

timestamp++

every cycle.

Document wraparound behavior.

Do not assume timestamps never overflow.

1.7 Software reference model

Implement the same page metadata logic in Rust.

Example:

struct PageMetadata {
    page_id
    request_id
    access_count
    last_access
    refcount
    valid
}

The Rust model is the golden reference.

1.8 Differential testing

Generate random events:

READ
READ
RELEASE
READ
ALLOCATE
...

Feed the same events to:

Rust model
Verilator model

Compare metadata after every event.

If they disagree:

FAIL

and print:

event index
event
expected metadata
actual metadata
1.9 Edge cases

Test:

counter overflow
refcount overflow
refcount underflow
timestamp wrap
reset during operation
invalid page
duplicate allocation
release of invalid page

The expected behavior must be defined before implementation.

1.10 Deliverables
[ ] single-page metadata RTL
[ ] event interface
[ ] timestamp counter
[ ] access counter
[ ] refcount
[ ] valid/state tracking
[ ] Rust reference model
[ ] differential tests
[ ] random tests
[ ] waveform examples
[ ] protocol documentation