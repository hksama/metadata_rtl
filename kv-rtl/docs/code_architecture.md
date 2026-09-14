# Documentation Index

Overview of the KV-RTL project and pointers to detailed architecture docs.
Also includes detailed characterstics of things implemented and how to run different processes.
---

## Project layout

```text
metadata_rtl/                    ← git repo root
├── checkpoint_0.md … checkpoint_3.md   ← milestone specifications
├── design_principles.md
└── kv-rtl/                      ← main project
    ├── rtl/                     ← SystemVerilog (hardware)
    ├── sim/                     ← C++ Verilator wrapper (C ABI)
    ├── rust/                    ← reference model, tests, evaluation
    ├── tests/cpp/               ← C++ smoke tests
    ├── traces/synthetic/        ← generated traces, CSVs, waveforms
    ├── scripts/                 ← verilate.sh, simulate.sh, clean.sh
    ├── docs/                    ← this folder
    └── Makefile
```

---

## Architecture documents

| Document | Read this for… |
|----------|----------------|
| **[rust_architecture.md](rust_architecture.md)** | Rust crate structure, modules, data flow, binaries, testing |
| **[rtl_architecture.md](rtl_architecture.md)** | SystemVerilog modules, ports, parameters, pipeline |
| **[rtl/README.md](rtl/README.md)** | Per-file SV reference (`kv_pkg`, hotness engine, metadata engine, `top`) |
| **[implementation_map.md](implementation_map.md)** | What is in Rust vs RTL vs not built yet |
| [architecture.md](architecture.md) | Short CP2 stack diagram (legacy overview) |
| [hotness_policies.md](hotness_policies.md) | CP3 hotness policies and Makefile knobs |
| [protocol.md](protocol.md) | Event opcodes, hit/miss/error, 2-cycle timing |
| [decisions.md](decisions.md) | CP2 design decision log |

---

## Quick answers

**What does the hardware do?**  
Tracks up to `NUM_PAGES` KV pages, accepts allocate/access/release events, reports
hit/miss/error, and outputs an integer hotness score (Checkpoint 3).

**What does Rust do?**  
Mirrors hardware for verification, generates synthetic workloads, and runs offline
policy evaluation (oracle, metrics, CSV sweeps).

**What is not built yet?**  
Eviction, cache simulation, runtime integration, real LLM traces, synthesis.
See [implementation_map.md](implementation_map.md).

**How do I run everything?**

```bash
cd kv-rtl
make test           # verilate + C++ smoke + Rust differential tests
make policy-test    # policy evaluation CSV
make simulate       # generate checkpoint2.vcd waveform
```

---

 What policy is running right now?

  There are two different “policies” in play:

  1. Hardware (RTL / Verilator) — one policy per build

  Default: FREQUENCY (exact access-count hotness).

  Set in Makefile and scripts/verilate.sh:

   kv-rtl/Makefile lines 8-11

  POLICY          ?= FREQUENCY
  SCORE_WIDTH     ?= 8
  AGING_INTERVAL  ?= 64
  NUM_PAGES       ?= 16

  When you run make verilate or make test, Verilator builds with whatever POLICY was set. The build prints it:

  Verilator model built: POLICY=FREQUENCY SCORE_WIDTH=8 AGING_INTERVAL=64

  Valid POLICY names: FREQUENCY, AGING2, AGING4, LRU (aliases like FREQ, AGING_2BIT, APPROX_LRU also work in verilate.sh).

  2. Rust differential tests — must match RTL

  make test runs kv-rtl-harness, which uses:

  let policy_config = PolicyConfig::default();  // FREQUENCY, score_width=8, aging_interval=64

  So make test assumes RTL was built with POLICY=FREQUENCY (defaults). If you rebuild RTL with POLICY=AGING2 but don’t change the Rust
  harness config, differential tests will fail.

  3. Policy evaluation (make policy-test) — all 4 policies in software only

  kv-eval --all-policies runs FREQUENCY, AGING2, AGING4, and LRU through the Rust MetadataTable only. It does not re-verilate or test
  RTL for each policy. Output goes to checkpoint3_policy_eval.csv.

  ────────────────────────────────────────

  Build / run parameters

  Makefile parameters (RTL compile-time)

  ┌────────────────┬───────────┬──────────────────────────────────┬───────────────────────────────────────────────────┐
  │ Parameter      │ Default   │ Example                          │ Effect                                            │
  ├────────────────┼───────────┼──────────────────────────────────┼───────────────────────────────────────────────────┤
  │ POLICY         │ FREQUENCY │ POLICY=AGING2 make verilate      │ Hotness algorithm in silicon model                │
  ├────────────────┼───────────┼──────────────────────────────────┼───────────────────────────────────────────────────┤
  │ SCORE_WIDTH    │ 8         │ SCORE_WIDTH=12 make verilate     │ Hotness score bit width                           │
  ├────────────────┼───────────┼──────────────────────────────────┼───────────────────────────────────────────────────┤
  │ AGING_INTERVAL │ 64        │ AGING_INTERVAL=256 make verilate │ Cycles between global aging ticks (AGING2/4 only) │
  ├────────────────┼───────────┼──────────────────────────────────┼───────────────────────────────────────────────────┤
  │ NUM_PAGES      │ 16        │ NUM_PAGES=64 make verilate       │ Metadata table size                               │
  └────────────────┴───────────┴──────────────────────────────────┴───────────────────────────────────────────────────┘

  Rust types.rs has NUM_PAGES = 16 — must match the Verilator build.

  Other environment variables

  ┌────────────────────┬──────────────────────────┬───────────────────────────────────────┐
  │ Variable           │ Used by                  │ Purpose                               │
  ├────────────────────┼──────────────────────────┼───────────────────────────────────────┤
  │ KV_RTL_WAVE=<path> │ make simulate, harness   │ Write VCD/FST waveform                │
  ├────────────────────┼──────────────────────────┼───────────────────────────────────────┤
  │ WAVE_FMT=fst       │ simulate.sh, verilate.sh │ FST instead of VCD                    │
  ├────────────────────┼──────────────────────────┼───────────────────────────────────────┤
  │ VERILATOR_INCLUDE  │ build scripts            │ Verilator include path if non-default │
  └────────────────────┴──────────────────────────┴───────────────────────────────────────┘

  ────────────────────────────────────────

  Commands you can run

  All from kv-rtl/:

  ┌──────────────────┬──────────────────────────────────────────────────────────────────────┐
  │ Command          │ What it does                                                         │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make test        │ Verilate + C++ smoke + Rust unit tests + full harness (RTL vs model) │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make verilate    │ Rebuild Verilator model only                                         │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make cpp-test    │ C++ smoke test on RTL                                                │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make rust-test   │ cargo test + kv-rtl-harness                                          │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make policy-test │ Software eval of all 4 policies → checkpoint3_policy_eval.csv        │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make aging-sweep │ AGING2/4 × 7 intervals → checkpoint3_aging_sweep.csv                 │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make simulate    │ C++ tb + Rust harness with waveform → checkpoint2.vcd                │
  ├──────────────────┼──────────────────────────────────────────────────────────────────────┤
  │ make clean       │ Wipe build/ artifacts                                                │
  └──────────────────┴──────────────────────────────────────────────────────────────────────┘

  Direct Cargo:

  cd kv-rtl/rust
  cargo test --release
  cargo run --release --bin kv-rtl-harness
  cargo run --release --bin kv-eval -- --all-policies
  cargo run --release --bin kv-eval -- --aging-sweep

  Waveform example:

  KV_RTL_WAVE=traces/synthetic/my_run.vcd make test
  gtkwave traces/synthetic/my_run.vcd

  ────────────────────────────────────────

  Files in traces/synthetic/

  CSV files

  ┌─────────────────────────┬────────────────────────────┬────────────────────────────────────────────────────────────────────────────┐
  │ File                    │ Produced by                │ Contents                                                                   │
  ├─────────────────────────┼────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ checkpoint3_metrics.csv │ make test (harness)        │ Single run stats: events, cycles, hits, misses, allocations, releases,     │
  │                         │                            │ errors                                                                     │
  ├─────────────────────────┼────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ checkpoint3_policy_eval │ make test and make         │ All policies × workloads: precision/recall, Spearman, Kendall, top-K,      │
  │ .csv                    │ policy-test                │ metadata bits                                                              │
  ├─────────────────────────┼────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ checkpoint3_aging_sweep │ make aging-sweep           │ AGING2 and AGING4 at intervals 16, 32, 64, …, 1024                         │
  │ .csv                    │                            │                                                                            │
  ├─────────────────────────┼────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ checkpoint2_metrics.csv │ Older CP2 run              │ Same shape as checkpoint3_metrics (legacy name)                            │
  └─────────────────────────┴────────────────────────────┴────────────────────────────────────────────────────────────────────────────┘

  checkpoint3_metrics.csv example:

  metric,value
  events,200
  cycles,600
  cycles_per_event,3.0000
  hits,68
  misses,84
  ...

  checkpoint3_policy_eval.csv columns:

  policy, workload, hot_precision, hot_recall, cold_precision, cold_recall, spearman, kendall, top_k_overlap, metadata_bits, logic_ops,
  state_bits

  Workloads include: sequential, uniform_random, single_hot, multiple_hot, bursty, zipf_alpha_*, agentic.

  Text files

  ┌───────────────────────┬─────────────┬─────────────────────────────────────────────────────────────────────────┐
  │ File                  │ Produced by │ Format                                                                  │
  ├───────────────────────┼─────────────┼─────────────────────────────────────────────────────────────────────────┤
  │ checkpoint3_trace.txt │ make test   │ One event per line: A = allocate, C = access, R = release, then page ID │
  ├───────────────────────┼─────────────┼─────────────────────────────────────────────────────────────────────────┤
  │ checkpoint2_trace.txt │ Older run   │ Same format (legacy)                                                    │
  └───────────────────────┴─────────────┴─────────────────────────────────────────────────────────────────────────┘

  Example:

  C 83      ← access page 83
  A 9       ← allocate page 9
  R 9       ← release page 9

  These are human-readable listings of a sample synthetic workload, not input to the RTL directly.

  Waveform

  ┌─────────────────┬───────────────┬─────────────────────────────────────────────────────────┐
  │ File            │ Produced by   │ Tool                                                    │
  ├─────────────────┼───────────────┼─────────────────────────────────────────────────────────┤
  │ checkpoint2.vcd │ make simulate │ Open in GTKWave (name is legacy; content is CP3 design) │
  └─────────────────┴───────────────┴─────────────────────────────────────────────────────────┘

  ────────────────────────────────────────

  How to run and check different parameters

  Change RTL policy (hardware sim)

  cd kv-rtl
  make clean
  POLICY=AGING2 AGING_INTERVAL=128 make verilate
  make cpp-test

  Check build line: POLICY=AGING2 ....

  Important: Rust differential tests still use PolicyConfig::default() (FREQUENCY). For a full make test pass with AGING2 RTL, the
  harness would need to use the same policy (today it doesn’t expose that via env/Makefile).

  Change table size

  NUM_PAGES=64 make clean verilate

  Also update rust/src/types.rs NUM_PAGES to 64 (they must match).

  Run all policies (software evaluation only)

  make policy-test
  cat traces/synthetic/checkpoint3_policy_eval.csv

  Compare rows by policy and workload columns.

  Sweep aging intervals

  make aging-sweep
  cat traces/synthetic/checkpoint3_aging_sweep.csv

  Look at workload column values like AGING2_interval_64.

  Inspect a single run’s hit/miss stats

  make test
  cat traces/synthetic/checkpoint3_metrics.csv

  View waves for default FREQUENCY build

  make simulate
  gtkwave traces/synthetic/checkpoint2.vcd

  Useful signals: top.event_valid, top.resp_valid, top.resp_hit, top.resp_hotness_score, top.hw_timestamp.

  ────────────────────────────────────────

  Quick mental model

  make test
    └── RTL: whatever POLICY you built (default FREQUENCY)
    └── Rust model for diff tests: FREQUENCY (PolicyConfig::default())
    └── Also writes: checkpoint3_metrics.csv, checkpoint3_trace.txt, checkpoint3_policy_eval.csv
  make policy-test / aging-sweep
    └── Rust only — all policies / aging sweeps
    └── Does not change or verify RTL per policy
  make simulate
    └── Same RTL as last verilate
    └── Writes checkpoint2.vcd

## Current milestone

**Checkpoint 3** — hotness tracking policies (frequency, 2-bit/4-bit aging, approx LRU).

Next: **Checkpoint 4** — candidate selection and eviction.

See [../KV_RTL_MVP_ROADMAP.md](../KV_RTL_MVP_ROADMAP.md) for the full roadmap.
