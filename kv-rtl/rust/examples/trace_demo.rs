use kv_rtl::{tick_idle, AccessEvent, MetadataSimulator, Operation, VerilatorSimulator};

fn main() {
    let wave = std::env::var("KV_RTL_WAVE")
        .unwrap_or_else(|_| "traces/synthetic/checkpoint2.vcd".into());
    let mut sim = VerilatorSimulator::with_trace(&wave).expect("enable trace");

    for page_id in [10u16, 11, 12] {
        let alloc = AccessEvent {
            request_id: 1,
            page_id,
            timestamp: page_id,
            operation: Operation::Allocate,
        };
        sim.drive_event(&alloc);
        sim.tick();
        sim.deassert_event();
        sim.tick();
        tick_idle(&mut sim);
    }

    for page_id in [10u16, 11] {
        let access = AccessEvent {
            request_id: 1,
            page_id,
            timestamp: 100 + page_id,
            operation: Operation::Access,
        };
        sim.drive_event(&access);
        sim.tick();
        sim.deassert_event();
        sim.tick();
        tick_idle(&mut sim);
    }

    sim.finalize_trace();
    println!("Wrote checkpoint 2 demo waveform to {wave}");
}
