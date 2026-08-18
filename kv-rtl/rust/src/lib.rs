//! KV-RTL library: simulator abstraction, reference model, trace generation, metrics.

pub mod eval;
pub mod metrics;
pub mod model;
pub mod oracle;
pub mod policy;
pub mod trace_gen;
pub mod types;

mod verilator;

pub use eval::{evaluate_aging_sweep, evaluate_all_policies, evaluate_policy_on_trace};
pub use metrics::{
    build_policy_eval_result, classification_metrics, evaluate_at_position, kendall_correlation,
    spearman_correlation, top_k_overlap, write_policy_eval_csv, ClassificationMetrics, Metrics,
    PolicyEvalResult, RankingMetrics,
};
pub use model::{EventResult, MetadataTable, PageTracker};
pub use oracle::{future_hot_labels, future_hotness_scores, future_reuse_counts};
pub use policy::{
    estimate_hardware_cost, policy_score, policy_update_aging, refresh_scores, should_age_tick,
    HardwareCost, PolicyConfig,
};
pub use trace_gen::{
    all_workload_patterns, generate_trace, write_trace_listing, TraceConfig, TracePattern,
    AGING_INTERVALS, ZIPF_ALPHAS,
};
pub use types::{
    AccessEvent, HotnessPolicy, Operation, PageMetadata, PageState, COUNTER_MAX, DEFAULT_AGING_INTERVAL,
    DEFAULT_SCORE_WIDTH, NUM_PAGES, REFCOUNT_MAX, TIMESTAMP_MAX, WIDTHS,
};
pub use verilator::VerilatorSimulator;

/// Hardware simulator interface for the metadata table engine.
pub trait MetadataSimulator {
    fn reset(&mut self);
    fn tick(&mut self);
    fn drive_event(&mut self, event: &AccessEvent);
    fn deassert_event(&mut self);
    fn resp_valid(&self) -> bool;
    fn resp_error(&self) -> bool;
    fn resp_hit(&self) -> bool;
    fn resp_miss(&self) -> bool;
    fn resp_entry_id(&self) -> Option<u8>;
    fn resp_metadata(&self) -> PageMetadata;
    fn resp_hotness_score(&self) -> u8;
    fn table_entries(&self) -> Vec<PageMetadata>;
    fn hw_timestamp(&self) -> u16;
    fn cycle_count(&self) -> u64;
    fn enable_trace(&mut self, path: &str) -> Result<(), &'static str>;
    fn finalize_trace(&mut self);
}

/// Submit one access event and wait for the two-cycle accept/commit handshake.
pub fn submit_event(sim: &mut dyn MetadataSimulator, event: &AccessEvent) -> EventResult {
    let start = sim.cycle_count();
    sim.drive_event(event);
    sim.tick();
    sim.deassert_event();
    sim.tick();
    let _ = start;
    EventResult {
        resp_valid: sim.resp_valid(),
        resp_error: sim.resp_error(),
        resp_hit: sim.resp_hit(),
        resp_miss: sim.resp_miss(),
        resp_entry_id: sim.resp_entry_id(),
        resp_metadata: sim.resp_metadata(),
        resp_hotness_score: sim.resp_hotness_score(),
    }
}

/// Advance one idle cycle (no event).
pub fn tick_idle(sim: &mut dyn MetadataSimulator) {
    sim.deassert_event();
    sim.tick();
}
