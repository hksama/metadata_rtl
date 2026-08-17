//! Simulator abstraction — Rust test/runtime sits above this layer, not Verilator internals.

mod verilator;
pub use verilator::VerilatorSimulator;

/// Hardware simulator interface used by tests and future runtime code.
pub trait Simulator {
    fn reset(&mut self);
    fn tick(&mut self);
    fn drive_input(&mut self, valid: bool, data: u32);
    fn output_valid(&self) -> bool;
    fn output_data(&self) -> u32;
    fn cycle_count(&self) -> u64;
    fn enable_trace(&mut self, path: &str) -> Result<(), &'static str>;
    fn finalize_trace(&mut self);
}

/// Run one idle cycle with inputs deasserted.
pub fn tick_idle(sim: &mut dyn Simulator) {
    sim.drive_input(false, 0);
    sim.tick();
}

/// Drive a single transaction and return `(output_valid, output_data, cycles_elapsed)`.
pub fn push_and_wait(sim: &mut dyn Simulator, data: u32) -> (bool, u32, u64) {
    let start = sim.cycle_count();
    sim.drive_input(true, data);
    sim.tick();
    sim.drive_input(false, 0);
    sim.tick();
    let valid = sim.output_valid();
    let out = sim.output_data();
    let cycles = sim.cycle_count() - start;
    (valid, out, cycles)
}
