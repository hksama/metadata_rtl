 What make simulate does (3 steps)

  make simulate
  # → scripts/simulate.sh

  ┌───────────────────────────────────────────────────────┐
  │                                                       │
  │                 1. scripts/verilate.sh                │
  │                                                       │
  └───────────────────────────┬───────────────────────────┘
                              ▼
  ┌───────────────────────────────────────────────────────┐
  │                                                       │
  │        Compile RTL → build/tb_top + libkvsim.a        │
  │                                                       │
  └───────────────────────────┬───────────────────────────┘
                              ▼
  ┌───────────────────────────────────────────────────────┐
  │                                                       │
  │              2. build/tb_top (C++ smoke)              │
  │                                                       │
  └───────────────────────────┬───────────────────────────┘
                              ▼
  ┌───────────────────────────────────────────────────────┐
  │                                                       │
  │ 3. KV_RTL_WAVE=... cargo run --release (Rust harness) │
  │                                                       │
  └───────────────────────────┬───────────────────────────┘
                              ▼
  ┌───────────────────────────────────────────────────────┐
  │                                                       │
  │                checkpoint2.vcd written                │
  │                                                       │
  └───────────────────────────────────────────────────────┘
  ctrl+o to show source

  Step 1 — Verilate (scripts/verilate.sh)

  Rebuilds the Verilator model from:

  • rtl/kv_pkg.sv
  • rtl/kv_hotness_engine.sv
  • rtl/kv_metadata_engine.sv
  • rtl/top.sv

  Compile-time knobs from the environment (Makefile defaults):

  ┌────────────────┬───────────┬───────────────────┐
  │ Variable       │ Default   │ Effect            │
  ├────────────────┼───────────┼───────────────────┤
  │ POLICY         │ FREQUENCY │ -GPOLICY=0..3     │
  ├────────────────┼───────────┼───────────────────┤
  │ SCORE_WIDTH    │ 8         │ -GSCORE_WIDTH=    │
  ├────────────────┼───────────┼───────────────────┤
  │ AGING_INTERVAL │ 64        │ -GAGING_INTERVAL= │
  ├────────────────┼───────────┼───────────────────┤
  │ NUM_PAGES      │ 16        │ -GNUM_PAGES=      │
  └────────────────┴───────────┴───────────────────┘

  Also builds build/tb_top (C++ smoke binary) and build/libkvsim.a (simulator wrapper).

  Step 2 — C++ smoke test (build/tb_top)

  Runs a tiny hardcoded sequence — not a trace file:

   kv-rtl/tests/cpp/tb_top.cpp lines 37-42

      drive(sim, 3, 1, 10, 0);  // ALLOCATE page 10
      submit(sim);
      ...
      drive(sim, 4, 1, 10, 5);  // ACCESS page 10
      ...
      drive(sim, 4, 1, 99, 6);  // ACCESS miss

  Each event uses the 2-cycle protocol: drive → tick → deassert → tick (submit()).

  This step:

  • Does not read any trace file
  • Does not write a waveform
  • Uses a separate sim_create() instance from step 3
  • Assumes FREQUENCY (checks hotness_score == access_count)

  Step 3 — Rust harness with tracing

   kv-rtl/scripts/simulate.sh lines 22-24

  echo "Running Rust harness (waveform -> ${WAVE_FILE})..."
  cd "${ROOT}/rust"
  KV_RTL_WAVE="${WAVE_FILE}" cargo run --release

  When KV_RTL_WAVE is set, the harness opens a new traced simulator:

   kv-rtl/rust/src/main.rs lines 17-22

      let policy_config = PolicyConfig::default();
      let mut sim = if let Ok(wave_path) = std::env::var("KV_RTL_WAVE") {
          VerilatorSimulator::with_trace(&wave_path).map_err(|e| e.to_string())?
      } else {
          VerilatorSimulator::new()
      };

  Waveform output:

  • Default: traces/synthetic/checkpoint2.vcd (legacy name)
  • FST: WAVE_FMT=fst make simulate → checkpoint2.fst

  At the end: sim.finalize_trace() closes the VCD/FST file.

  ────────────────────────────────────────

  Where trace data comes from (important)

  make simulate does not load trace files from disk. There is no parser for checkpoint3_trace.txt or any CSV.

  All events are either hardcoded in C++ or generated in memory in Rust via generate_trace().

  What actually drives the VCD

  One long Verilator session runs these phases back-to-back on the same DUT (with one reset at the start):

  ┌───────────────────────────────┬───────────────────────┬────────┬────────────────────────────────────────────────────────────────┐
  │ Phase                         │ Source                │ Events │ Pattern / seed                                                 │
  ├───────────────────────────────┼───────────────────────┼────────┼────────────────────────────────────────────────────────────────┤
  │ run_table_edge_tests          │ Hardcoded in main.rs  │ ~20+   │ Allocs, hits, miss, fill table, alloc-when-full error, release │
  ├───────────────────────────────┼───────────────────────┼────────┼────────────────────────────────────────────────────────────────┤
  │ run_hotness_policy_tests      │ Hardcoded + new reset │ 6      │ Alloc page 42, 5 accesses                                      │
  ├───────────────────────────────┼───────────────────────┼────────┼────────────────────────────────────────────────────────────────┤
  │ run_random_differential_tests │ generate_trace()      │ 300    │ TracePattern::Random, seed 0xC0FF_EE02                         │
  ├───────────────────────────────┼───────────────────────┼────────┼────────────────────────────────────────────────────────────────┤
  │ run_workload_with_metrics     │ generate_trace()      │ 200    │ TracePattern::Mixed, seed 0xBEE_FOOD                           │
  └───────────────────────────────┴───────────────────────┴────────┴────────────────────────────────────────────────────────────────┘

  Each event goes through the 2-cycle pipeline:

   kv-rtl/rust/src/main.rs lines 148-158

  fn apply_and_compare(...) {
      sim.drive_event(&event);
      let _latch = model.step(Some(event));
      sim.tick();
      sim.deassert_event();
      let commit = model.step(None);
      sim.tick();
      compare_state(...)
  }

  After each event in the metrics phase, one idle cycle is inserted (tick_idle) so global aging ticks can fire on idle cycles.

  What is not in the VCD

  These are written after simulation, in software only — never replayed through Verilator:

  ┌─────────────────────────────┬───────────────────────────────────────────────────────┬─────────────────────────────────┐
  │ Artifact                    │ How produced                                          │ In VCD?                         │
  ├─────────────────────────────┼───────────────────────────────────────────────────────┼─────────────────────────────────┤
  │ checkpoint3_trace.txt       │ generate_trace() — 50 events, seed 0x5151_0001, Mixed │ No                              │
  ├─────────────────────────────┼───────────────────────────────────────────────────────┼─────────────────────────────────┤
  │ checkpoint3_metrics.csv     │ Metrics from the 200-event Mixed run above            │ Partially reflects VCD workload │
  ├─────────────────────────────┼───────────────────────────────────────────────────────┼─────────────────────────────────┤
  │ checkpoint3_policy_eval.csv │ evaluate_all_policies() — pure Rust MetadataTable     │ No                              │
  └─────────────────────────────┴───────────────────────────────────────────────────────┴─────────────────────────────────┘

  So if you open checkpoint3_trace.txt and expect to see those exact events in the VCD, they won’t match. The VCD is the concatenation
  of edge tests + hotness test + 300 random + 200 mixed events.

  ────────────────────────────────────────

  How events are driven (mechanism)

  Rust AccessEvent  →  C ABI KvEvent  →  top.event_* ports  →  kv_metadata_engine

  C++ layer (sim/simulator.cpp):

  • sim_drive_event() — sets event_valid, page_id, operation, etc.
  • sim_tick() — one full clock (low→high), dumps VCD sample if tracing
  • sim_deassert_event() — clears inputs for commit cycle
  • sim_read_table_entry() — combinational peek via table_read/table_idx (sim-only port)

  Rust compares every cycle RTL vs MetadataTable (reference model): responses, full table state, hw_timestamp, resp_hotness_score.

  ────────────────────────────────────────

  What policy make simulate uses

  RTL (what you actually simulate)

  Whatever was compiled in step 1:

  make simulate                    # POLICY=FREQUENCY (default)
  POLICY=AGING2 make simulate      # RTL built with AGING2
  POLICY=LRU AGING_INTERVAL=256 make simulate

  Only one policy per Verilator build. No runtime switching in SV.

  Rust harness (reference model for diff tests)

  Always:

   kv-rtl/rust/src/main.rs lines 17-17

      let policy_config = PolicyConfig::default();

  That is FREQUENCY, score_width=8, aging_interval=64.

  Consequence

  ┌─────────────────────────────┬────────────────────────────────────────────────────────────────────────────────┐
  │ Command                     │ Result                                                                         │
  ├─────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ make simulate               │ Works — RTL and Rust both FREQUENCY                                            │
  ├─────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ POLICY=AGING2 make simulate │ Likely fails at compare_state on resp_hotness_score / table mismatch           │
  ├─────────────────────────────┼────────────────────────────────────────────────────────────────────────────────┤
  │ run_hotness_policy_tests    │ Resets sim and uses FREQUENCY model explicitly; still assumes RTL is FREQUENCY │
  └─────────────────────────────┴────────────────────────────────────────────────────────────────────────────────┘

  Policy evaluation CSV (evaluate_all_policies) runs all four policies in Rust only after the sim finishes. That does not touch the
  waveform or RTL.

  ────────────────────────────────────────

  What make simulate supports vs doesn’t

  Supports

  ┌────────────────────────────────────────────────────────┬──────────────────────────────────────────────────────────────────────────┐
  │ Feature                                                │ How                                                                      │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Cycle-accurate RTL sim                                 │ Verilator + simulator.cpp                                                │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ VCD waveform                                           │ KV_RTL_WAVE → sim_create_traced()                                        │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ FST waveform                                           │ WAVE_FMT=fst make simulate                                               │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Differential RTL vs Rust                               │ Full harness compare                                                     │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Compile-time POLICY, NUM_PAGES, SCORE_WIDTH,           │ Via make POLICY=... simulate (RTL only)                                  │
  │ AGING_INTERVAL                                         │                                                                          │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Global aging in RTL                                    │ If AGING2/4 built and Rust model matches (currently only at FREQUENCY    │
  │                                                        │ default)                                                                 │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ Idle cycles for aging ticks                            │ tick_idle() between Mixed workload events                                │
  ├────────────────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────┤
  │ C++ smoke before Rust                                  │ Quick sanity check                                                       │
  └────────────────────────────────────────────────────────┴──────────────────────────────────────────────────────────────────────────┘

  Does not support

  ┌───────────────────────────────────────┬────────────────────────────────────────────────────────────────────────────┐
  │ Limitation                            │ Detail                                                                     │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ No trace file input                   │ Cannot point at checkpoint3_trace.txt or external CSV                      │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ No workload CLI                       │ Cannot choose Zipf/bursty/agentic for the waveform without editing main.rs │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ No multi-policy RTL run               │ One policy per build; simulate doesn’t sweep policies through hardware     │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ Rust policy not tied to POLICY=       │ Harness always FREQUENCY unless code/env wiring is added                   │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ Oracle / Spearman / policy CSV in sim │ Those are offline Rust (kv-eval); not in SV or VCD flow                    │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ Cache hit rate / eviction             │ Not CP3; not implemented anywhere                                          │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ tb_top not in VCD                     │ Separate sim instance; waveform is Rust harness only                       │
  ├───────────────────────────────────────┼────────────────────────────────────────────────────────────────────────────┤
  │ Reading back written trace txt        │ write_trace_listing is export-only                                         │
  └───────────────────────────────────────┴────────────────────────────────────────────────────────────────────────────┘

  ────────────────────────────────────────

  Checkpoint 3: implemented in SV vs not

  Based on checkpoint_3.md and the actual RTL:

  Implemented in SystemVerilog

  ┌─────────────────────────────────────────────────┬─────────────────────────────────────────────────────────────────────────────────┐
  │ CP3 section                                     │ SV implementation                                                               │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ 3.1 Hotness engine separated from metadata      │ kv_hotness_engine.sv × NUM_PAGES; table stores fields, engine scores            │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ 3.2–3.6 Four policies                           │ kv_pkg::policy_update_aging, policy_score; FREQUENCY, AGING2, AGING4,           │
  │                                                 │ APPROX_LRU                                                                      │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ 3.7 Common policy_update / policy_score         │ Functions in kv_pkg.sv; engine calls them                                       │
  │ interface                                       │                                                                                 │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ 3.8 Integer scores, parameterized SCORE_WIDTH   │ Truncation in kv_hotness_engine; no floating point                              │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ 3.9 Compile-time policy selection               │ -GPOLICY= via Makefile / verilate.sh                                            │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ Metadata fields                                 │ access_count, last_access, aging_counter, refcount, etc. in                     │
  │                                                 │ kv_metadata_engine.sv                                                           │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ Configurable aging interval                     │ AGING_INTERVAL parameter; global tick on hw_timestamp % interval                │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ Event protocol + 2-cycle pipeline               │ Unchanged from CP2; scores on resp_hotness_score                                │
  ├─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
  │ Simulation debug                                │ table_read / table_idx combinational peek                                       │
  └─────────────────────────────────────────────────┴─────────────────────────────────────────────────────────────────────────────────┘

  Not in SystemVerilog (CP3 spec — Rust / tooling only)

  ┌───────────────────────────────────┬──────────────────────────┬────────────────────────────────────────────────────────────────────┐
  │ CP3 section                       │ Where it lives           │ Notes                                                              │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.10 Synthetic workloads          │ rust/src/trace_gen.rs    │ Sequential, random, hot, bursty, Zipf, etc. — events generated in  │
  │                                   │                          │ software                                                           │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.11 Agentic workload             │ trace_gen.rs             │ Same                                                               │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.12 Ground-truth oracle          │ rust/src/oracle.rs       │ Future reuse lookahead — impossible in real-time HW                │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.13 Classification metrics       │ rust/src/metrics.rs,     │ hot/cold precision/recall                                          │
  │                                   │ eval.rs                  │                                                                    │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.13 Ranking metrics              │ eval.rs                  │ Spearman, Kendall, top-K vs oracle                                 │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.13 Cache impact metrics         │ —                        │ Not implemented (needs CP4 cache sim)                              │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.13 Hardware cost estimates      │ rust/src/policy.rs       │ Rough bit/op model — not synthesizable logic                       │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ 3.14 Aging interval               │ make aging-sweep /       │ RTL holds one interval per build; sweep is Rust-only               │
  │ experiments/sweeps                │ eval.rs                  │                                                                    │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ Policy eval CSV export            │ kv-eval, harness         │ Not RTL                                                            │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ Runtime policy switching          │ —                        │ Explicitly deferred in spec                                        │
  ├───────────────────────────────────┼──────────────────────────┼────────────────────────────────────────────────────────────────────┤
  │ ML / FP predictors                │ —                        │ Explicitly out of scope                                            │
  └───────────────────────────────────┴──────────────────────────┴────────────────────────────────────────────────────────────────────┘

  CP3 items partially covered

  ┌────────────────────────────────┬────────────────────────┬─────────────────────────────────────────────────────────────┐
  │ Item                           │ SV                     │ Gap                                                         │
  ├────────────────────────────────┼────────────────────────┼─────────────────────────────────────────────────────────────┤
  │ Aging experiments (3.14)       │ Tick logic exists      │ No automated RTL sweep; change AGING_INTERVAL and rebuild   │
  ├────────────────────────────────┼────────────────────────┼─────────────────────────────────────────────────────────────┤
  │ Differential verification      │ RTL scores implemented │ Harness only validates against Rust when both use FREQUENCY │
  ├────────────────────────────────┼────────────────────────┼─────────────────────────────────────────────────────────────┤
  │ make simulate as “policy demo” │ RTL can be any policy  │ Waveform + diff tests assume FREQUENCY in Rust              │
  └────────────────────────────────┴────────────────────────┴─────────────────────────────────────────────────────────────┘

  ────────────────────────────────────────

  Practical mental model

  make simulate =
    rebuild RTL (one compile-time policy)
    + 4 hardcoded C++ events (no wave)
    + long Rust session driving RTL in-memory traces (THIS is the VCD)
    + write unrelated 50-event txt + offline policy eval CSV in Rust

  To inspect behavior in GTKWave:

  make simulate
  gtkwave traces/synthetic/checkpoint2.vcd

  Useful signals: top.event_valid, operation, page_id, resp_valid, resp_hit, resp_miss, resp_hotness_score, resp_meta_*, hw_timestamp,
  resp_meta_aging_counter.

  To simulate a non-default RTL policy with a matching reference today, you’d need the harness to read POLICY from the environment (or
  pass PolicyConfig accordingly) — that wiring is not there yet, which is why POLICY=AGING2 make simulate is expected to fail
  differential checks even though the SV policy code exists.