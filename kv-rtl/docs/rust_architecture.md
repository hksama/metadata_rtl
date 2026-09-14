# Rust Codebase Architecture

This document describes the Rust side of KV-RTL: how modules connect, what each
file does, and how data flows through simulation and evaluation.

For hardware details see [rtl_architecture.md](rtl_architecture.md).  
For what lives in Rust vs SystemVerilog vs neither, see [implementation_map.md](implementation_map.md).

---

## Role of Rust in the project

Rust is **not** the production runtime (that comes in Checkpoint 6+). Today it provides:

1. **Golden reference model** — software copy of RTL behavior for differential testing
2. **Verilator harness** — drives the simulated hardware via a C ABI
3. **Synthetic workloads** — generates access traces for experiments
4. **Offline evaluation** — oracle, metrics, policy sweeps (Checkpoint 3)
5. **Build integration** — `cargo` links against the Verilated model

```text
┌─────────────────────────────────────────────────────────────┐
│                     Rust workspace (rust/)                   │
├─────────────────────────────────────────────────────────────┤
│  Binaries          │  kv-rtl-harness (main.rs)              │
│                    │  kv-eval (bin/kv-eval.rs)              │
│                    │  trace_demo (examples/)                │
├────────────────────┼────────────────────────────────────────┤
│  Core library      │  types, model, policy                  │
│  (kv_rtl crate)   │  trace_gen, oracle, eval, metrics     │
├────────────────────┼────────────────────────────────────────┤
│  Simulation glue   │  verilator.rs + lib.rs trait           │
│                    │  build.rs → links libkvsim.a           │
└────────────────────┴────────────────────────────────────────┘
                              │
                              ▼
                    sim/simulator.cpp (C ABI)
                              │
                              ▼
                    Verilator Vtop (SystemVerilog)
```

---

## Crate layout

| Path | Kind | Purpose |
|------|------|---------|
| `rust/src/lib.rs` | Library root | Module exports, `MetadataSimulator` trait, `submit_event()` |
| `rust/src/types.rs` | Core | Widths, opcodes, `PageMetadata`, saturation helpers |
| `rust/src/model.rs` | Core | `MetadataTable` golden reference (2-cycle pipeline) |
| `rust/src/policy.rs` | Core | Hotness policy math (mirrors `kv_pkg.sv`) |
| `rust/src/trace_gen.rs` | Core | Synthetic workload generators |
| `rust/src/oracle.rs` | Eval only | Future-reuse ground truth (not in RTL) |
| `rust/src/eval.rs` | Eval only | Policy evaluation harness |
| `rust/src/metrics.rs` | Eval only | Counters, classification, ranking, CSV |
| `rust/src/verilator.rs` | Glue | FFI to `sim/simulator.h` |
| `rust/src/main.rs` | Binary | Differential tests + artifact generation |
| `rust/src/bin/kv-eval.rs` | Binary | Policy/aging sweep CLI |
| `rust/build.rs` | Build | Invokes `make verilate`, links static libs |
| `rust/examples/trace_demo.rs` | Example | Minimal waveform demo |

---

## Module dependency graph

```text
types.rs
   │
   ├── policy.rs ──────────────┐
   │                           │
   ├── model.rs ◄──────────────┘
   │       │
   │       └── used by eval.rs, main.rs (differential tests)
   │
   ├── trace_gen.rs
   │       │
   │       └── used by eval.rs, main.rs
   │
   ├── oracle.rs
   │       │
   │       └── used by eval.rs, metrics.rs
   │
   ├── metrics.rs ◄── policy.rs (hardware cost)
   │       │
   │       └── used by eval.rs, main.rs
   │
   └── eval.rs ◄── model, policy, trace_gen, oracle, metrics

verilator.rs ◄── types, MetadataSimulator trait in lib.rs
lib.rs ◄── all public modules + verilator
```

---

## Core types (`types.rs`)

Shared encodings that **must match** `rtl/kv_pkg.sv`:

| Type / constant | Role |
|-----------------|------|
| `NUM_PAGES` | Table size (default 16; must match Verilator `-GNUM_PAGES`) |
| `WIDTHS` | Field bit widths (page_id, counter, score, …) |
| `HotnessPolicy` | FREQUENCY, AGING2, AGING4, LRU (IDs 0–3) |
| `Operation` | READ, WRITE, RELEASE, ALLOCATE, ACCESS |
| `AccessEvent` | One input transaction to the engine |
| `PageMetadata` | One table slot including `aging_counter`, `hotness_score` |
| `sat_inc_*` / `sat_dec_*` | Saturating arithmetic matching RTL |

---

## Reference model (`model.rs`)

`MetadataTable` is the **authoritative software spec** of hardware behavior.

### Pipeline model

Mirrors the 2-cycle RTL handshake:

```text
step(Some(event))  →  cycle N: latch event into pending_event
step(None)         →  cycle N+1: commit, return EventResult
```

`submit_event()` = both steps in sequence (what tests call).

### State

| Field | Meaning |
|-------|---------|
| `entries[NUM_PAGES]` | Full metadata table |
| `hw_timestamp` | Free-running counter (+1 per step) |
| `policy_config` | Active hotness policy |
| `pending_event` | Latched but uncommitted event |

### Operations implemented

| Operation | Behavior |
|-----------|----------|
| `Allocate` | Lowest free slot; error if duplicate or full |
| `Read/Write/Access` | Hit: update count, last_access, aging, score; Miss: no table change |
| `Release` | Decrement refcount; error if missing or refcount=0 |

Also models **global aging ticks** on idle cycles (aging policies only).

---

## Hotness policies (`policy.rs`)

Software mirror of RTL `policy_update_aging()` and `policy_score()`:

| Function | RTL equivalent |
|----------|----------------|
| `policy_update_aging()` | `kv_pkg::policy_update_aging` |
| `policy_score()` | `kv_pkg::policy_score` |
| `should_age_tick()` | `global_age_tick` in metadata engine |
| `refresh_scores()` | Re-score all entries after aging |
| `estimate_hardware_cost()` | **Rust only** — rough bit/op estimates |

`PolicyConfig` holds `policy`, `score_width`, `aging_interval`.

---

## Workload generation (`trace_gen.rs`)

Produces `Vec<AccessEvent>` for simulation and evaluation.

| Pattern | Purpose |
|---------|---------|
| `Sequential` | Scan pattern — tests false hotness |
| `UniformRandom` | No locality baseline |
| `SingleHot` | One dominant page |
| `MultipleHot` | Several hot pages |
| `Bursty` | Shifting hot sets — recency/aging test |
| `Zipf` | Skewed popularity (α = 0.5, 1.0, 1.5, 2.0) |
| `Agentic` | Shared prefix + per-request suffixes |
| `Mixed` / `HotSpot` | Checkpoint 2 compatibility |

Trace generation is **Rust-only** — hardware never sees pattern names, only events.

---

## Oracle (`oracle.rs`) — evaluation only

Looks ahead in a trace to compute **ground-truth future reuse**:

| Function | Output |
|----------|--------|
| `future_reuse_counts()` | How many times each page is accessed in next N events |
| `future_hot_labels()` | Binary hot/cold per page |
| `future_hotness_scores()` | Continuous reuse counts for ranking |

**Not implemented in RTL.** Used only to score how well policies predict the future.

---

## Evaluation (`eval.rs` + `metrics.rs`)

### Flow

```text
generate_trace(config)
        │
        ▼
MetadataTable::submit_event() for each event
        │
        ├── record Metrics (hits, misses, cycles, …)
        │
        └── at each access: compare predicted hotness vs oracle
                │
                ├── ClassificationMetrics (precision/recall)
                └── RankingMetrics (Spearman, Kendall, top-K)
                        │
                        ▼
                PolicyEvalResult → CSV
```

### Entry points

| Function / binary | What it runs |
|-------------------|--------------|
| `evaluate_policy_on_trace()` | One policy × one trace |
| `evaluate_all_policies()` | 4 policies × 7 workloads (+ Zipf alphas) |
| `evaluate_aging_sweep()` | AGING2/4 × 7 intervals on bursty trace |
| `kv-eval --all-policies` | CLI wrapper |
| `kv-eval --aging-sweep` | CLI wrapper |

---

## Simulation glue

### `MetadataSimulator` trait (`lib.rs`)

Abstract interface so tests can target Verilator (or a future mock):

- Drive events, tick clock, read responses and full table state
- `submit_event()` implements the standard 2-tick handshake

### `VerilatorSimulator` (`verilator.rs`)

Thin FFI wrapper over `sim/simulator.h`:

```text
Rust AccessEvent  →  KvEvent struct  →  Vtop inputs
Vtop outputs      →  KvMetadata      →  PageMetadata
```

### `build.rs`

1. Runs `make verilate` if `build/verilator/Vtop__ALL.a` missing
2. Links `libkvsim.a`, `Vtop__ALL.a`, `libverilated.a`

---

## Binaries and how to run

```bash
cd kv-rtl

# Full differential test + artifact generation
make test
# internally: cargo run --release --bin kv-rtl-harness

# Policy evaluation sweeps
make policy-test
make aging-sweep

# Unit tests only
cd rust && cargo test --release
```

### `kv-rtl-harness` (`main.rs`)

1. Table edge-case differential tests (RTL vs model)
2. Hotness policy smoke test
3. 300-event random differential test
4. 200-event mixed workload + metrics CSV
5. Full policy evaluation CSV

### `kv-eval` (`bin/kv-eval.rs`)

Standalone policy/aging experiments without RTL differential tests.

---

## What Rust does **not** do (yet)

See [implementation_map.md](implementation_map.md) for the full list. Highlights:

- No eviction or cache simulation
- No real LLM trace ingestion
- No runtime integration with an inference engine
- No synthesis or timing analysis
- Oracle and ranking metrics are offline analysis only

---

## Related docs

| Document | Contents |
|----------|----------|
| [rtl_architecture.md](rtl_architecture.md) | SystemVerilog modules and ports |
| [implementation_map.md](implementation_map.md) | Rust vs RTL vs not implemented |
| [protocol.md](protocol.md) | Event/response timing and opcodes |
| [hotness_policies.md](hotness_policies.md) | Policy formulas and Makefile knobs |
