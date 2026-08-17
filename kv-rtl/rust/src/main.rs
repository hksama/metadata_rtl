use kv_rtl::{
    tick_idle, AccessEvent, MetadataSimulator, Operation, PageMetadata, PageTracker,
    VerilatorSimulator,
};

fn main() {
    if let Err(msg) = run() {
        eprintln!("FAIL: {msg}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut sim = if let Ok(wave_path) = std::env::var("KV_RTL_WAVE") {
        VerilatorSimulator::with_trace(&wave_path).map_err(|e| e.to_string())?
    } else {
        VerilatorSimulator::new()
    };

    run_edge_case_tests(&mut sim)?;
    run_random_differential_tests(&mut sim, 500, 0xC0FF_EE01)?;

    if std::env::var("KV_RTL_WAVE").is_ok() {
        sim.finalize_trace();
        println!(
            "Waveform written (inspect with gtkwave): {}",
            std::env::var("KV_RTL_WAVE").unwrap_or_default()
        );
    }

    println!("PASS: checkpoint 1 single-page metadata tests");
    Ok(())
}

fn sync_reset(model: &mut PageTracker, sim: &mut VerilatorSimulator) {
    model.reset();
    sim.reset();
}

fn compare_state(
    index: usize,
    event: Option<&AccessEvent>,
    model: &PageTracker,
    sim: &VerilatorSimulator,
    model_result: kv_rtl::model::EventResult,
) -> Result<(), String> {
    if sim.resp_valid() != model_result.resp_valid {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_valid mismatch"));
    }
    if sim.resp_error() != model_result.resp_error {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_error mismatch"));
    }
    if sim.metadata() != model.metadata {
        return Err(format_mismatch(index, event, model, sim, model_result, "metadata mismatch"));
    }
    if sim.hw_timestamp() != model.hw_timestamp {
        return Err(format_mismatch(index, event, model, sim, model_result, "hw_timestamp mismatch"));
    }
    Ok(())
}

fn format_mismatch(
    index: usize,
    event: Option<&AccessEvent>,
    model: &PageTracker,
    sim: &VerilatorSimulator,
    model_result: kv_rtl::EventResult,
    reason: &str,
) -> String {
    format!(
        "event_index={index}\n\
         reason={reason}\n\
         event={event:?}\n\
         expected_metadata={:?}\n\
         actual_metadata={:?}\n\
         expected_hw_timestamp={}\n\
         actual_hw_timestamp={}\n\
         expected_resp_valid={} actual_resp_valid={}\n\
         expected_resp_error={} actual_resp_error={}",
        model.metadata,
        sim.metadata(),
        model.hw_timestamp,
        sim.hw_timestamp(),
        model_result.resp_valid,
        sim.resp_valid(),
        model_result.resp_error,
        sim.resp_error(),
    )
}

fn apply_and_compare(
    index: usize,
    model: &mut PageTracker,
    sim: &mut VerilatorSimulator,
    event: AccessEvent,
) -> Result<(), String> {
    sim.drive_event(&event);
    let _latch = model.step(Some(event));
    sim.tick();

    sim.deassert_event();
    let commit = model.step(None);
    sim.tick();

    compare_state(index, Some(&event), model, sim, commit)
}

fn run_edge_case_tests(sim: &mut VerilatorSimulator) -> Result<(), String> {
    let mut model = PageTracker::new();
    sync_reset(&mut model, sim);

    let page = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 5,
        operation: Operation::Allocate,
    };
    apply_and_compare(0, &mut model, sim, page)?;

    let read = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 99,
        operation: Operation::Read,
    };
    apply_and_compare(1, &mut model, sim, read)?;

    let write = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 100,
        operation: Operation::Write,
    };
    apply_and_compare(2, &mut model, sim, write)?;

    let bad_page = AccessEvent {
        request_id: 42,
        page_id: 999,
        timestamp: 1,
        operation: Operation::Read,
    };
    apply_and_compare(3, &mut model, sim, bad_page)?;

    let dup_alloc = AccessEvent {
        request_id: 1,
        page_id: 173,
        timestamp: 2,
        operation: Operation::Allocate,
    };
    apply_and_compare(4, &mut model, sim, dup_alloc)?;

    let release = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 3,
        operation: Operation::Release,
    };
    apply_and_compare(5, &mut model, sim, release)?;

    let underflow_release = AccessEvent {
        request_id: 42,
        page_id: 173,
        timestamp: 4,
        operation: Operation::Release,
    };
    apply_and_compare(6, &mut model, sim, underflow_release)?;

    // access_count saturation (reference model unit path + RTL near-max spot check)
    sync_reset(&mut model, sim);
    let alloc = AccessEvent {
        request_id: 1,
        page_id: 10,
        timestamp: 0,
        operation: Operation::Allocate,
    };
    apply_and_compare(7, &mut model, sim, alloc)?;

    for ts in 0..4 {
        let event = AccessEvent {
            request_id: 1,
            page_id: 10,
            timestamp: ts,
            operation: Operation::Read,
        };
        apply_and_compare(8 + ts as usize, &mut model, sim, event)?;
    }
    assert_eq!(model.metadata.access_count, 4);

    // hw_timestamp increments on idle cycles (wrap tested in unit tests)
    sync_reset(&mut model, sim);
    for _ in 0..8 {
        tick_idle(sim);
        model.step(None);
        if sim.hw_timestamp() != model.hw_timestamp || sim.metadata() != model.metadata {
            return Err(format_mismatch(20, None, &model, sim, kv_rtl::EventResult { resp_valid: false, resp_error: false }, "idle timestamp drift"));
        }
    }

    // reset during pending operation
    sync_reset(&mut model, sim);
    let pending = AccessEvent {
        request_id: 1,
        page_id: 55,
        timestamp: 7,
        operation: Operation::Allocate,
    };
    sim.drive_event(&pending);
    model.step(Some(pending));
    sim.tick();
    sim.deassert_event();
    sim.reset();
    model.reset();
    if sim.metadata() != PageMetadata::INVALID {
        return Err("metadata not cleared after reset during operation".into());
    }

    // release on invalid page
    let release_invalid = AccessEvent {
        request_id: 1,
        page_id: 1,
        timestamp: 1,
        operation: Operation::Release,
    };
    apply_and_compare(21, &mut model, sim, release_invalid)?;

    Ok(())
}

fn run_random_differential_tests(
    sim: &mut VerilatorSimulator,
    count: usize,
    seed: u64,
) -> Result<(), String> {
    let mut model = PageTracker::new();
    sync_reset(&mut model, sim);

    let mut rng = LcgRng::new(seed);
    let page_id = (rng.next_u16() % 1024) + 1;
    let request_id = (rng.next_u16() % 1024) + 1;

    for index in 0..count {
        let op = match rng.next_u32() % 5 {
            0 => Operation::Read,
            1 => Operation::Write,
            2 => Operation::Release,
            3 => Operation::Allocate,
            _ => {
                sim.deassert_event();
                sim.tick();
                let result = model.step(None);
                compare_state(index, None, &model, sim, result)?;
                continue;
            }
        };

        let event = AccessEvent {
            request_id,
            page_id,
            timestamp: rng.next_u16(),
            operation: op,
        };
        apply_and_compare(index, &mut model, sim, event)?;
    }

    Ok(())
}

struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (self.state >> 16) as u32
    }

    fn next_u16(&mut self) -> u16 {
        (self.next_u32() & 0xFFFF) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kv_rtl::{COUNTER_MAX, PageState, REFCOUNT_MAX};

    #[test]
    fn differential_random_smoke() {
        let mut sim = VerilatorSimulator::new();
        run_random_differential_tests(&mut sim, 100, 1).expect("random diff failed");
    }

    #[test]
    fn edge_cases() {
        let mut sim = VerilatorSimulator::new();
        run_edge_case_tests(&mut sim).expect("edge cases failed");
    }

    #[test]
    fn refcount_saturation_unit() {
        assert_eq!(REFCOUNT_MAX.saturating_add(1), REFCOUNT_MAX);
    }

    #[test]
    fn access_count_saturation_reference_model() {
        let mut model = PageTracker::new();
        model.set_metadata_for_test(
            PageMetadata {
                page_id: 1,
                request_id: 1,
                access_count: COUNTER_MAX,
                last_access: 0,
                refcount: 1,
                valid: true,
                state: kv_rtl::PageState::Valid,
            },
            0,
        );
        let event = AccessEvent {
            request_id: 1,
            page_id: 1,
            timestamp: 9,
            operation: Operation::Read,
        };
        assert!(!model.submit_event(event).resp_error);
        assert_eq!(model.metadata.access_count, COUNTER_MAX);
    }

    #[test]
    fn hw_timestamp_wraps_reference_model() {
        let mut model = PageTracker::new();
        model.hw_timestamp = u16::MAX;
        model.step(None);
        assert_eq!(model.hw_timestamp, 0);
    }
}
