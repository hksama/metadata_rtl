//! KV-RTL library: simulator abstraction, reference model, and shared types.

pub mod model;
pub mod types;

mod verilator;

pub use model::{EventResult, PageTracker};
pub use types::{AccessEvent, Operation, PageMetadata, PageState, WIDTHS, COUNTER_MAX, REFCOUNT_MAX, TIMESTAMP_MAX};
pub use verilator::VerilatorSimulator;

/// Hardware simulator interface for the single-page metadata tracker.
pub trait MetadataSimulator {
    fn reset(&mut self);
    fn tick(&mut self);
    fn drive_event(&mut self, event: &AccessEvent);
    fn deassert_event(&mut self);
    fn resp_valid(&self) -> bool;
    fn resp_error(&self) -> bool;
    fn metadata(&self) -> PageMetadata;
    fn hw_timestamp(&self) -> u16;
    fn cycle_count(&self) -> u64;
    fn enable_trace(&mut self, path: &str) -> Result<(), &'static str>;
    fn finalize_trace(&mut self);
}

/// Submit one access event and wait for the two-cycle accept/commit handshake.
pub fn submit_event(sim: &mut dyn MetadataSimulator, event: &AccessEvent) -> bool {
    sim.drive_event(event);
    sim.tick();
    sim.deassert_event();
    sim.tick();
    sim.resp_valid()
}

/// Advance one idle cycle (no event) and return the hardware timestamp.
pub fn tick_idle(sim: &mut dyn MetadataSimulator) -> u16 {
    sim.deassert_event();
    sim.tick();
    sim.hw_timestamp()
}
