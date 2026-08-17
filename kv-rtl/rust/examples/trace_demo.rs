use kv_rtl::{push_and_wait, tick_idle, VerilatorSimulator};
use kv_rtl::Simulator;

fn main() {
    let wave = std::env::var("KV_RTL_WAVE").unwrap_or_else(|_| "traces/synthetic/checkpoint0.vcd".into());
    let mut sim = VerilatorSimulator::with_trace(&wave).expect("enable trace");

    for value in [10u32, 20, 30] {
        let (valid, out, _) = push_and_wait(&mut sim, value);
        assert!(valid);
        assert_eq!(out, value.wrapping_add(1));
        tick_idle(&mut sim);
    }

    sim.finalize_trace();
    println!("Wrote demo waveform to {wave}");
}
