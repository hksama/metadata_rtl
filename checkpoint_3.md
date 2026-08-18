3.1 Architecture

The architecture must become:

                    ACCESS EVENT
                         │
                         ▼
                ┌─────────────────┐
                │ Metadata Table  │
                │                 │
                │ page_id         │
                │ request_id      │
                │ refcount        │
                │ access_count    │
                │ last_access     │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │ Hotness Engine  │
                │                 │
                │ policy selected │
                └────────┬────────┘
                         │
                         ▼
                    HOTNESS SCORE

The hotness engine must be logically separated from metadata storage.

This is important because later policies may be implemented independently.

3.2 Supported policies

Implement the following in stages:

Exact frequency
2-bit aging
4-bit aging
Approximate LRU

Do NOT initially implement:

floating-point scores
machine learning predictors
neural predictors
complicated probabilistic algorithms
3.3 Policy 1 — Exact frequency

For every page:

hotness = access_count

Example:

Page A → 100 accesses
Page B → 20 accesses
Page C → 5 accesses

Result:

A > B > C

Advantages:

very cheap
simple
easy to verify

Disadvantages:

does not account for recency
old accesses remain important forever
3.4 Policy 2 — 2-bit aging

Add:

aging_counter[1:0]

Example:

0 = cold
1 = warm
2 = hot
3 = very hot

On access:

counter = min(counter + 1, 3)

At an aging interval:

counter = counter >> 1

The aging interval must be configurable.

Example:

AGING_INTERVAL = 64 cycles
AGING_INTERVAL = 256 cycles
AGING_INTERVAL = 1024 cycles

Do NOT hard-code one interval.

3.5 Policy 3 — 4-bit aging

Same concept:

aging_counter[3:0]

Possible range:

0 → 15

Compare against 2-bit aging.

Measure:

metadata bits
logic
update cost
ranking quality
3.6 Policy 4 — Approximate LRU

Use:

last_access_timestamp

and calculate:

age = current_timestamp - last_access_timestamp

A page with larger age is colder.

Do not initially implement an exact LRU linked list.

The purpose is to establish a recency baseline.

3.7 Hardware policy interface

Design a common interface:

policy_update(
    access_event,
    metadata
)


policy_score(
    metadata
)

Conceptually:

                 metadata
                    │
                    ▼
              ┌───────────┐
              │  POLICY   │
              └─────┬─────┘
                    │
                    ▼
                 score

The rest of the RTL must not care whether the policy is:

frequency
aging
LRU
3.8 Score representation

Use integer/fixed-point representation.

Initially:

score_width = 8 bits

but parameterize it.

Investigate:

4-bit
8-bit
12-bit
16-bit

Do not use floating point.

Document why.

3.9 Policy configuration

Support compile-time selection initially:

POLICY_FREQUENCY
POLICY_AGING_2BIT
POLICY_AGING_4BIT
POLICY_APPROX_LRU

Example:

make POLICY=FREQUENCY
make POLICY=AGING2
make POLICY=AGING4
make POLICY=LRU

Runtime policy switching can be introduced later.

3.10 Synthetic workloads

Create a workload generator capable of producing:

Sequential
0 1 2 3 4 5 6 7 ...

Purpose:

test whether policies incorrectly identify sequential scans as hot.

Uniform random
7 2 15 4 8 1 13 6 ...

Purpose:

minimal locality baseline.

Single hot page
1 1 1 1 1 1 1 4 7 3

Purpose:

strong locality.

Multiple hot pages
1 2 1 3 2 1 3 2 1 3

Purpose:

multiple simultaneously hot prefixes.

Bursty
A A A A A
        ↓
B B B B B
        ↓
C C C C C
        ↓
A A A A A

Purpose:

test recency adaptation.

Zipf

Generate page accesses according to a Zipf distribution.

Parameters:

alpha = 0.5
alpha = 1.0
alpha = 1.5
alpha = 2.0

Purpose:

approximate skewed real-world cache access.

3.11 Agent/LLM-inspired synthetic workload

Create a workload resembling agentic inference:

system prefix
    │
    ├── request A
    ├── request B
    ├── request C
    │
    ▼
shared prefix

Then:

shared prefix
+
different suffixes

Represent this only as KV page IDs.

Do NOT put natural-language semantics into RTL.

3.12 Ground-truth policy

For small experiments, implement an offline reference.

Given a future access window:

current time = T
lookahead = N accesses

calculate whether each page will be reused.

This allows:

predicted hotness
        vs
future reuse

to be compared.

This does NOT need to become part of RTL.

It is only an evaluation oracle.

3.13 Metrics

Measure:

Classification
hot-page precision
hot-page recall
cold-page precision
cold-page recall
Ranking

For candidate pages:

Spearman correlation
Kendall correlation
top-K overlap

against the offline reference.

Cache impact

When connected to a simulated cache:

cache hit rate
eviction rate
false eviction rate
re-fetch rate
Hardware cost

Estimate:

metadata bits/page
logic operations/access
comparators
adders
subtractors
state bits
3.14 Aging experiments

For each policy:

2-bit aging
4-bit aging

vary:

aging interval:
16
32
64
128
256
512
1024

Determine:

accuracy
hardware cost
adaptation speed