 There are four hotness policies in the SystemVerilog design. They share the same metadata table and event protocol; only the score 
  formula and aging behavior differ. Rust mirrors those same four policies for verification and offline evaluation, and adds Rust-only
  oracle/metrics tooling on top.

  ────────────────────────────────────────

  The four SV policies

  Defined in kv_pkg.sv as compile-time IDs:

  ┌──────────────┬──────────────────┬─────┬───────────────────────────────────────┬───────────────────────────────────────────────────┐
  │ Makefile     │ SV constant      │ ID  │ Hotness score                         │ Extra metadata behavior                           │
  │ name         │                  │     │                                       │                                                   │
  ├──────────────┼──────────────────┼─────┼───────────────────────────────────────┼───────────────────────────────────────────────────┤
  │ FREQUENCY    │ POLICY_FREQUENCY │ 0   │ access_count (truncated to            │ No aging counter updates                          │
  │              │                  │     │ SCORE_WIDTH)                          │                                                   │
  ├──────────────┼──────────────────┼─────┼───────────────────────────────────────┼───────────────────────────────────────────────────┤
  │ AGING2       │ POLICY_AGING_2BI │ 1   │ aging_counter[1:0]                    │ On access: counter += 1, max 3; global tick: >> 1 │
  │              │ T                │     │                                       │                                                   │
  ├──────────────┼──────────────────┼─────┼───────────────────────────────────────┼───────────────────────────────────────────────────┤
  │ AGING4       │ POLICY_AGING_4BI │ 2   │ aging_counter[3:0]                    │ On access: counter += 1, max 15; global tick: >>  │
  │              │ T                │     │                                       │ 1                                                 │
  ├──────────────┼──────────────────┼─────┼───────────────────────────────────────┼───────────────────────────────────────────────────┤
  │ LRU          │ POLICY_APPROX_LR │ 3   │ 65535 - (now - last_access)           │ Uses last_access; no aging counter                │
  │              │ U                │     │                                       │                                                   │
  └──────────────┴──────────────────┴─────┴───────────────────────────────────────┴───────────────────────────────────────────────────┘

  Core logic in kv_pkg.sv:

   kv-rtl/rtl/kv_pkg.sv lines 95-123

      function automatic logic [AGING_COUNTER_MAX_WIDTH-1:0] policy_update_aging(
          ...
          if (do_age) begin
              next = aging_in >> 1;
          end else if (is_access) begin
              unique case (policy)
                  POLICY_AGING_2BIT: next = sat_inc_aging(aging_in, 4'd3);
                  POLICY_AGING_4BIT: next = sat_inc_aging(aging_in, 4'd15);
                  default: next = aging_in;
              endcase
          end
          ...
      function automatic logic [DEFAULT_SCORE_WIDTH-1:0] policy_score(
          ...
                  unique case (policy)
                      POLICY_FREQUENCY: begin
                          raw = access_count[DEFAULT_SCORE_WIDTH-1:0];
                      end
                      POLICY_AGING_2BIT: begin
                          raw = {{(DEFAULT_SCORE_WIDTH - 2){1'b0}}, aging_counter[1:0]};
                      end
                      POLICY_AGING_4BIT: begin
                          raw = {{(DEFAULT_SCORE_WIDTH - 4){1'b0}}, aging_counter[3:0]};
                      end
                      POLICY_APPROX_LRU: begin
                          age = current_timestamp - last_access;
                          recency = {{(TIMESTAMP_WIDTH){1'b1}}} - age;
                          raw = recency[DEFAULT_SCORE_WIDTH-1:0];
                      end

  How each policy behaves (intuitively)

  1. FREQUENCY — “How many times was this page accessed?” Higher count → hotter. Simple and good when hot pages are accessed often.

  2. AGING2 / AGING4 — “How recently active is this page?” Each access bumps a small counter; every AGING_INTERVAL cycles (when
     hw_timestamp % AGING_INTERVAL == 0), all non-accessed entries’ counters are halved. Score is the counter value. 2-bit is coarser
     (0–3); 4-bit has more gradation (0–15).

  3. APPROX_LRU — “How long since last access?” Score is recency: larger means more recently touched. No global aging tick;
     last_access is updated on each hit.

  All policies: integer only, no floating point; invalid entries score 0.

  ────────────────────────────────────────

  How to run policies in hardware (RTL)

  Policy is compile-time via Verilator -GPOLICY=. The Makefile passes it through scripts/verilate.sh:

  cd kv-rtl
  # Default: FREQUENCY
  make verilate
  # Pick a policy
  make POLICY=AGING2 verilate
  make POLICY=AGING4 AGING_INTERVAL=256 verilate
  make POLICY=LRU verilate
  # Other knobs
  make POLICY=AGING2 SCORE_WIDTH=12 NUM_PAGES=64 verilate

  ┌──────────────────┬──────────────────────────────────────────────────────────────────┐
  │ Target           │ What it does with policy                                         │
  ├──────────────────┼──────────────────────────────────────────────────────────────────┤
  │ make test        │ Verilates (with POLICY), C++ smoke test, Rust differential tests │
  ├──────────────────┼──────────────────────────────────────────────────────────────────┤
  │ make simulate    │ Verilates + runs harness with VCD waveform                       │
  ├──────────────────┼──────────────────────────────────────────────────────────────────┤
  │ make policy-test │ Verilates once, then Rust-only eval of all 4 policies            │
  ├──────────────────┼──────────────────────────────────────────────────────────────────┤
  │ make aging-sweep │ Verilates once, then Rust sweep of aging intervals               │
  └──────────────────┴──────────────────────────────────────────────────────────────────┘

  Important mismatch: make test and the harness always use PolicyConfig::default() → FREQUENCY in Rust, while RTL uses whatever you
  passed to make POLICY=.... So:

  • make test — OK (both default to FREQUENCY)
  • POLICY=AGING2 make test — will likely fail differential tests unless the harness is wired to read POLICY from the environment

  To exercise a non-default policy in RTL today, rebuild with that POLICY, then drive events yourself (C++ tb_top, or a custom
  harness). Waveforms:

  POLICY=AGING2 make simulate
  gtkwave traces/synthetic/checkpoint2.vcd

  Watch resp_hotness_score, resp_meta_aging_counter, resp_meta_access_count, hw_timestamp.

  ────────────────────────────────────────

  What test data exists (and what you can point at)

  There is no trace file loader in the repo today. All workloads are generated in Rust by trace_gen.rs (generate_trace()). You cannot
  point kv-eval or the harness at an arbitrary CSV/trace file without adding a parser.

  Generated workloads (trace_gen.rs)

  Used by evaluate_all_policies():

  ┌───────────────┬──────────────────────────┬──────────────────────────────────────┐
  │ Pattern       │ Name in CSV              │ Description                          │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ Sequential    │ sequential               │ Pages 0,1,2,…                        │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ UniformRandom │ uniform_random           │ Uniform random page IDs              │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ SingleHot     │ single_hot               │ One page dominates                   │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ MultipleHot   │ multiple_hot             │ Several hot pages                    │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ Bursty        │ bursty                   │ Bursts on A, then B, then C, …       │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ Zipf          │ zipf_alpha_{0.5,1,1.5,2} │ Zipf skew (4 alphas)                 │
  ├───────────────┼──────────────────────────┼──────────────────────────────────────┤
  │ Agentic       │ agentic                  │ Shared prefix + per-request suffixes │
  └───────────────┴──────────────────────────┴──────────────────────────────────────┘

  Default eval: 150 events, seed 0xC30013. Aging sweep uses bursty, 200 events, intervals [16, 32, 64, 128, 256, 512, 1024].

  Output artifacts (traces/synthetic/)

  ┌─────────────────────────────┬─────────────────────────────┬────────────────────────────────────────────────────────┐
  │ File                        │ Produced by                 │ Contents                                               │
  ├─────────────────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────┤
  │ checkpoint3_trace.txt       │ make test / harness         │ Human-readable: A alloc, C access, R release + page_id │
  ├─────────────────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────┤
  │ checkpoint3_metrics.csv     │ harness                     │ hits, misses, cycles, etc.                             │
  ├─────────────────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────┤
  │ checkpoint3_policy_eval.csv │ make policy-test or harness │ All policies × workloads vs oracle                     │
  ├─────────────────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────┤
  │ checkpoint3_aging_sweep.csv │ make aging-sweep            │ AGING2/4 × interval sweep                              │
  ├─────────────────────────────┼─────────────────────────────┼────────────────────────────────────────────────────────┤
  │ checkpoint2.vcd             │ make simulate               │ Waveform (legacy name; CP3 RTL inside)                 │
  └─────────────────────────────┴─────────────────────────────┴────────────────────────────────────────────────────────┘

  Example trace line format:

  C 83    # access page 83
  A 9     # allocate page 9
  R 9     # release page 9

  That .txt format is write-only today — there is no read_trace_listing().

  Commands to run policy evaluation (Rust, software model)

  cd kv-rtl
  make policy-test      # → traces/synthetic/checkpoint3_policy_eval.csv
  make aging-sweep      # → traces/synthetic/checkpoint3_aging_sweep.csv
  # Or directly:
  cd rust
  cargo run --release --bin kv-eval -- --all-policies
  cargo run --release --bin kv-eval -- --aging-sweep

  These replay traces through MetadataTable (Rust reference model), not through Verilator — except that make still runs verilate first
  as a dependency.

  ────────────────────────────────────────

  What is “pure Rust” vs what is in both

  Same four policies — in both RTL and Rust

  Rust HotnessPolicy mirrors the SV IDs exactly:

   kv-rtl/rust/src/types.rs lines 37-42

  pub enum HotnessPolicy {
      Frequency = 0,
      Aging2Bit = 1,
      Aging4Bit = 2,
      ApproxLru = 3,
  }

  policy.rs implements the same policy_update_aging() and policy_score() as kv_pkg.sv. model.rs uses them so differential tests can
  compare RTL vs software cycle-by-cycle.

  There are no extra hotness policies in Rust — only the same four, with Rust able to switch among them at runtime in eval.

  Rust-only (not in SV)

  ┌──────────────────────────────┬────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │ Component                    │ Role                                                                                               │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ oracle.rs                    │ Looks ahead in a trace: “which pages will be accessed in the next N events?” Ground truth for      │
  │                              │ evaluation; impossible in real-time hardware                                                       │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ eval.rs                      │ Replays traces, compares policy scores vs oracle (Spearman, Kendall, top-K, precision/recall)      │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ metrics.rs                   │ Aggregates classification/ranking metrics, writes CSV                                              │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ trace_gen.rs                 │ Synthetic workload generation                                                                      │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ policy.rs::estimate_hardware │ Rough metadata-bit and logic-op estimates per policy                                               │
  │ _cost()                      │                                                                                                    │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ kv-eval binary               │ CLI for --all-policies and --aging-sweep                                                           │
  ├──────────────────────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ Verilator harness (main.rs,  │ Drives RTL, compares to MetadataTable                                                              │
  │ verilator.rs)                │                                                                                                    │
  └──────────────────────────────┴────────────────────────────────────────────────────────────────────────────────────────────────────┘

  Key differences in how they run

  ┌───────────────────┬────────────────────────────────────────┬──────────────────────────────────────────────────┐
  │ Aspect            │ RTL (SV)                               │ Rust                                             │
  ├───────────────────┼────────────────────────────────────────┼──────────────────────────────────────────────────┤
  │ Policy selection  │ One policy at compile time (-GPOLICY=) │ All four at runtime in kv-eval                   │
  ├───────────────────┼────────────────────────────────────────┼──────────────────────────────────────────────────┤
  │ Purpose           │ Synthesizable hotness engine           │ Reference model + offline quality measurement    │
  ├───────────────────┼────────────────────────────────────────┼──────────────────────────────────────────────────┤
  │ Oracle / Spearman │ Not present                            │ Core of policy comparison                        │
  ├───────────────────┼────────────────────────────────────────┼──────────────────────────────────────────────────┤
  │ Trace input       │ Events via simulator ports             │ In-memory Vec<AccessEvent> from generate_trace() │
  ├───────────────────┼────────────────────────────────────────┼──────────────────────────────────────────────────┤
  │ Global aging      │ Hardware hw_timestamp % AGING_INTERVAL │ Same logic in should_age_tick()                  │
  └───────────────────┴────────────────────────────────────────┴──────────────────────────────────────────────────┘

  So: the policies themselves are not different between Rust and RTL — the formulas match. What differs is what Rust adds around them:
  oracle-based evaluation, sweeps, synthetic traces, and cost modeling, none of which exist in silicon.

  ────────────────────────────────────────

  Practical workflow summary

  ┌───────────────────────────────────────────────┬──────────────────────────────────────────────┐
  │ Goal                                          │ Command                                      │
  ├───────────────────────────────────────────────┼──────────────────────────────────────────────┤
  │ Verify RTL matches Rust (FREQUENCY)           │ make test                                    │
  ├───────────────────────────────────────────────┼──────────────────────────────────────────────┤
  │ Run RTL with AGING2 and capture waves         │ POLICY=AGING2 make simulate                  │
  ├───────────────────────────────────────────────┼──────────────────────────────────────────────┤
  │ Compare all 4 policies on synthetic workloads │ make policy-test                             │
  ├───────────────────────────────────────────────┼──────────────────────────────────────────────┤
  │ Tune aging interval for AGING2/4              │ make aging-sweep                             │
  ├───────────────────────────────────────────────┼──────────────────────────────────────────────┤
  │ Inspect results                               │ traces/synthetic/checkpoint3_policy_eval.csv │
  └───────────────────────────────────────────────┴──────────────────────────────────────────────┘

  If you want to feed your own trace data, you would need to either build Vec<AccessEvent> in Rust and call evaluate_policy_on_trace(),
  or add a trace parser — that path is not implemented yet. Switch to Agent mode if you want that wired up.