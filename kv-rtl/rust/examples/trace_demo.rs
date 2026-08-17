use kv_rtl::{tick_idle, AccessEvent, MetadataSimulator, Operation, VerilatorSimulator};

fn main() {
    let wave = std::env::var("KV_RTL_WAVE")
        .unwrap_or_else(|_| "traces/synthetic/checkpoint1.vcd".into());
    let mut sim = VerilatorSimulator::with_trace(&wave).expect("enable trace");

    let alloc = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 10,
        operation: Operation::Allocate,
    };
    sim.drive_event(&alloc);
    sim.tick();
    sim.deassert_event();
    sim.tick();

    for (ts, op) in [
        (20u16, Operation::Read),
        (21, Operation::Write),
        (22, Operation::Release),
    ] {
        let event = AccessEvent {
            request_id: 42,
            page_id: 173,
            timestamp: ts,
            operation: op,
        };
        sim.drive_event(&event);
        sim.tick();
        sim.deassert_event();
        sim.tick();
        tick_idle(&mut sim);
    }

    sim.finalize_trace();
    println!("Wrote checkpoint 1 demo waveform to {wave}");
}
