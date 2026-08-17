Principle 1 — Hardware must operate on KV pages/blocks

The RTL must NOT know about:

prompts
agent.md
source code
documents
users' semantic information
natural language

The RTL abstraction is:

request
page
access event
metadata
decision

Example:

REQUEST_ID = 42
PAGE_ID    = 173
ACCESS     = READ
Principle 2 — Separate policy from metadata storage

Do not hard-code one hotness algorithm into the entire architecture.

The architecture should eventually support:

EXACT_LRU
APPROX_LRU
FREQUENCY
2_BIT_AGING
4_BIT_AGING
CLOCK
SAMPLED
HYBRID

Initially only implement a small subset.

Principle 3 — Do not optimize communication yet

Early checkpoints should intentionally ignore:

PCIe latency
NVLink latency
InfiniBand
RDMA
CXL
GPU synchronization

These will be introduced later.

The first goal is to determine:

How much work can the metadata engine perform per cycle?

Only later determine whether that work is worth offloading.

Principle 4 — Every checkpoint must be executable

Each checkpoint must produce:

compilable RTL
Verilator simulation
automated tests
reference software implementation
benchmark output
documentation
reproducible commands

Never move to the next checkpoint if the current checkpoint cannot be
reproduced from a clean checkout.