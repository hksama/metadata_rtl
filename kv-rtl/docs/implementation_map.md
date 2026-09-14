# Implementation Map — Rust vs RTL vs Not Implemented

This document answers: **what exists in hardware**, **what exists only in Rust**,
**what exists in both** (and must stay in sync), and **what is not built yet**.

Use this as the single reference when deciding where to add a new feature.

---

## Summary diagram

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                         CHECKPOINT 3 SCOPE                               │
├──────────────────────────────┬──────────────────────────────────────────┤
│         HARDWARE (RTL)        │              RUST ONLY                    │
├──────────────────────────────┼──────────────────────────────────────────┤
│ Metadata table (NUM_PAGES)   │ Golden reference model (MetadataTable)    │
│ Allocate / Access / Release  │ Differential tests (RTL vs model)         │
│ Hit / Miss / Error responses │ Synthetic trace generation                │
│ 2-cycle event pipeline       │ Offline reuse oracle                      │
│ hw_timestamp counter         │ Policy quality metrics (Spearman, etc.)   │
│ 4 hotness policies           │ Hardware cost estimates                   │
│ Integer hotness scores       │ Policy/aging sweep automation             │
│ Global aging ticks           │ CSV / trace artifact export               │
│ Saturating counters          │ Verilator harness (via C ABI)           │
└──────────────────────────────┴──────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                      NOT IMPLEMENTED (ANYWHERE)                          │
├─────────────────────────────────────────────────────────────────────────┤
│ Eviction, victim selection, cache simulation, batched events,            │
│ runtime integration, real LLM traces, NUMA placement, synthesis,         │
│ runtime policy switching, ML predictors, floating-point hotness          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Feature matrix

Legend:

- **RTL** — in SystemVerilog (`rtl/*.sv`)
- **Rust** — in Rust library (`rust/src/`)
- **Both** — implemented in RTL and mirrored in Rust (must match)
- **—** — not implemented

| Feature | RTL | Rust | Notes |
|---------|-----|------|-------|
| **Infrastructure** |
| Verilator sim environment | via `sim/` | `build.rs`, `verilator.rs` | CP0 |
| C ABI simulator | `sim/simulator.cpp` | FFI wrapper | Glue layer |
| Makefile / build knobs | `scripts/verilate.sh` | — | POLICY, NUM_PAGES, etc. |
| **Metadata table** |
| Multi-page table (`NUM_PAGES`) | ✓ | ✓ (`model.rs`) | Both |
| Parallel page_id lookup | ✓ | ✓ (linear scan) | Same semantics |
| Lowest-index free slot alloc | ✓ | ✓ | Both |
| `page_id`, `request_id` storage | ✓ | ✓ | Both |
| `access_count` (saturating) | ✓ | ✓ | Both |
| `last_access` timestamp | ✓ | ✓ | Both |
| `refcount` (saturating) | ✓ | ✓ | Both |
| `valid` / `state` fields | ✓ | ✓ | Both |
| **Event protocol** |
| Opcodes READ/WRITE/RELEASE/ALLOCATE/ACCESS | ✓ | ✓ (`types.rs`) | Both |
| 2-cycle accept/commit pipeline | ✓ | ✓ (`step()`) | Both |
| `resp_hit` / `resp_miss` / `resp_error` | ✓ | ✓ | Both |
| Miss is not an error | ✓ | ✓ | Both |
| `hw_timestamp` (+1/cycle) | ✓ | ✓ | Both |
| **Hotness (CP3)** |
| Policy: exact frequency | ✓ | ✓ (`policy.rs`) | Both |
| Policy: 2-bit aging | ✓ | ✓ | Both |
| Policy: 4-bit aging | ✓ | ✓ | Both |
| Policy: approximate LRU | ✓ | ✓ | Both |
| `aging_counter` storage | ✓ | ✓ | Both |
| Global aging interval tick | ✓ | ✓ | Both |
| `hotness_score` on response | ✓ | ✓ | Both |
| Compile-time policy select | ✓ | ✓ (eval uses all) | RTL: one at build; Rust tests all |
| Integer scores (no FP) | ✓ | ✓ | By design |
| `SCORE_WIDTH` parameter | ✓ | ✓ | Both |
| **Testing & verification** |
| Differential RTL vs model | — | ✓ (`main.rs`) | Rust drives RTL |
| C++ smoke test | `tests/cpp/tb_top.cpp` | — | Minimal RTL check |
| Table debug read port | ✓ (`table_read`) | via simulator | Sim only |
| VCD waveform dump | via `sim/` | `KV_RTL_WAVE` env | Both trigger same sim |
| **Workloads & evaluation** |
| Synthetic trace: sequential | — | ✓ | Rust generates events |
| Synthetic trace: random | — | ✓ | |
| Synthetic trace: single/multiple hot | — | ✓ | |
| Synthetic trace: bursty | — | ✓ | |
| Synthetic trace: Zipf | — | ✓ | |
| Synthetic trace: agentic | — | ✓ | |
| Future-reuse oracle | — | ✓ (`oracle.rs`) | **Not in RTL** |
| Hot/cold classification metrics | — | ✓ (`metrics.rs`) | **Not in RTL** |
| Spearman / Kendall / top-K | — | ✓ | **Not in RTL** |
| Hardware cost estimates | — | ✓ (`policy.rs`) | **Not in RTL** |
| Policy eval CSV export | — | ✓ | **Not in RTL** |
| Aging interval sweep | — | ✓ (`eval.rs`) | Rust sweeps; RTL one interval/build |
| **Not implemented** |
| Eviction / victim selection | — | — | CP4 |
| Cache hit rate simulation | — | — | CP4+ |
| False eviction / re-fetch metrics | — | — | CP4+ |
| Batched async accelerator | — | — | CP5 |
| Inference runtime integration | — | — | CP6 |
| Real LLM KV traces | — | — | CP7 |
| NUMA / tiered memory | — | — | CP8 |
| HW/SW coordination model | — | — | CP9 |
| Synthesis / area / timing / power | — | — | CP10 |
| Runtime policy switching | — | — | Future |
| ML / neural hotness predictors | — | — | Out of scope |
| Floating-point hotness | — | — | Out of scope |
| Exact LRU linked list | — | — | Approx LRU only |
| Indexed/direct-mapped lookup | — | — | Parallel compare for now |

---

## Must stay in sync (Both)

These pairs **must produce identical results** or differential tests fail:

| RTL location | Rust location |
|--------------|---------------|
| `kv_pkg.sv` opcodes, widths | `types.rs` |
| `kv_pkg::sat_inc_u`, `sat_inc_ref`, `sat_dec_ref` | `sat_inc_u16`, `sat_inc_u8`, `sat_dec_u8` |
| `kv_pkg::policy_update_aging` | `policy::policy_update_aging` |
| `kv_pkg::policy_score` | `policy::policy_score` |
| `kv_metadata_engine` pipeline + table ops | `model::MetadataTable` |
| `NUM_PAGES`, policy at Verilate time | `types::NUM_PAGES`, harness config |

When changing behavior, update **both** sides and run `make test`.

---

## Rust-only rationale

| Component | Why not in RTL |
|-----------|----------------|
| `oracle.rs` | Needs future trace lookahead — impossible in real-time hardware |
| `eval.rs` classification/ranking | Research analysis, not part of accelerator |
| `trace_gen.rs` | Test bench artifact; hardware receives events, not patterns |
| `estimate_hardware_cost()` | Planning tool before synthesis |
| `MetadataTable` | Reference for verification; silicon uses registers |
| Differential tests | Verification methodology, not product logic |

---

## RTL-only rationale

| Component | Why not duplicated in Rust logic |
|-----------|----------------------------------|
| `kv_hotness_engine` module instances | Structural hardware; Rust uses functions |
| Combinational parallel lookup | Rust uses sequential scan (same result) |
| `table_read` debug port | Simulation backdoor only |

---

## Checkpoint progress

| Checkpoint | Status | RTL | Rust |
|------------|--------|-----|------|
| CP0 Infrastructure | Done | trivial top → full engine | harness + build |
| CP1 Single page | Done (superseded) | merged into multi-page | `PageTracker` alias |
| CP2 Multi-page table | Done | `kv_metadata_engine` | `model.rs`, `trace_gen` (partial) |
| CP3 Hotness policies | Done | `kv_hotness_engine`, policies | `policy.rs`, `eval.rs`, oracle |
| CP4 Eviction | **Not started** | — | — |
| CP5+ | **Not started** | — | — |

---

## Where to add new features

| If you want to… | Implement in… |
|-----------------|---------------|
| Change how pages are tracked | RTL + `model.rs` |
| Add a new hotness policy | `kv_pkg.sv` + `kv_hotness_engine.sv` + `policy.rs` |
| Score policies against future reuse | `oracle.rs`, `eval.rs` only |
| Add a new synthetic workload | `trace_gen.rs` only |
| Evict a page from cache | CP4 — new RTL module + model (not started) |
| Connect to inference runtime | CP6 — mostly Rust, some RTL MMIO later |
| Measure synthesis area | CP10 — RTL + external tools |

---

## Related docs

| Document | Contents |
|----------|----------|
| [rust_architecture.md](rust_architecture.md) | Rust module guide |
| [rtl_architecture.md](rtl_architecture.md) | SystemVerilog module guide |
| [protocol.md](protocol.md) | Event/response protocol |
| [hotness_policies.md](hotness_policies.md) | Policy formulas |
| [../KV_RTL_MVP_ROADMAP.md](../KV_RTL_MVP_ROADMAP.md) | Full checkpoint roadmap |
