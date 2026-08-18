Objective

Move from one page to a configurable collection of KV pages.

This is the first checkpoint where the design begins resembling a real
KV-cache manager.

2.1 Metadata table

Create:

NUM_PAGES

configurable at compile time.

Example:

16
64
256
1024

Each entry contains:

page_id
request_id
access_count
last_access
refcount
valid
state
2.2 Lookup

Given:

page_id

find its metadata entry.

Initially implement a simple lookup.

Possible implementation:

parallel comparison

or:

indexed lookup

The first implementation should prioritize correctness and clarity.

Do NOT prematurely optimize.

2.3 Allocation

Implement:

ALLOCATE(page_id, request_id)

The engine must find a free metadata entry.

Return:

success
entry_id

or:

failure
2.4 Access

Implement:

ACCESS(page_id, timestamp)

The engine must:

find the page
update access count
update last-access timestamp
return metadata
signal miss if page does not exist
2.5 Release

Implement:

RELEASE(page_id)

Update:

refcount

Do not allow negative reference counts.

2.6 Miss handling

Distinguish:

HIT
MISS

A miss is NOT automatically an eviction.

The metadata engine only reports:

page not tracked

at this stage.

2.7 Multiple operations

The design must define whether it supports:

one event/cycle

or:

multiple events/cycle

Start with:

one event/cycle

but architect the interface so multi-event processing can be added later.

2.8 Pipeline consideration

Document the critical path:

input
 ↓
lookup
 ↓
metadata read
 ↓
update
 ↓
output

Estimate:

latency
throughput

in cycles.

Do not claim physical timing yet.

Verilator gives functional cycle simulation, not real FPGA/ASIC timing.

2.9 Random workload generator

Generate traces containing:

allocations
accesses
releases
misses

Example:

A 10
A 11
A 12
R 10
R 11
A 13
R 12

Also generate realistic locality:

hot pages
warm pages
cold pages
sequential accesses
random accesses
bursty accesses
2.10 Metrics

Measure:

events processed
cycles
cycles/event
hits
misses
allocations
releases

Produce:

CSV

for later plotting.