//! KV-RTL library: simulator abstraction, reference model, trace generation, metrics.

pub mod metrics;
pub mod model;
pub mod trace_gen;
pub mod types;

mod verilator;

pub use metrics::Metrics;
pub use model::{EventResult, MetadataTable, PageTracker};
pub use trace_gen::{generate_trace, write_trace_listing, TraceConfig, TracePattern};
pub use types::{
    AccessEvent, Operation, PageMetadata, PageState, COUNTER_MAX, NUM_PAGES, REFCOUNT_MAX,
    TIMESTAMP_MAX, WIDTHS,
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
    }
}

/// Advance one idle cycle (no event).
pub fn tick_idle(sim: &mut dyn MetadataSimulator) {
    sim.deassert_event();
    sim.tick();
}
