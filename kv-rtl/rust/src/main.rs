use std::path::PathBuf;

use kv_rtl::{
    generate_trace, tick_idle, write_trace_listing, AccessEvent, EventResult, MetadataSimulator,
    MetadataTable, Metrics, Operation, TraceConfig, TracePattern, VerilatorSimulator, NUM_PAGES,
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

    run_table_edge_tests(&mut sim)?;
    run_random_differential_tests(&mut sim, 300, 0xC0FF_EE02)?;

    let metrics = run_workload_with_metrics(&mut sim, TracePattern::Mixed)?;
    let trace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("traces/synthetic/checkpoint2_trace.txt");
    let csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("traces/synthetic/checkpoint2_metrics.csv");

    let trace = generate_trace(&TraceConfig {
        seed: 0x5151_0001,
        num_events: 50,
        pattern: TracePattern::Mixed,
        ..TraceConfig::default()
    });
    write_trace_listing(&trace_path, &trace).map_err(|e| e.to_string())?;
    metrics.write_csv(&csv_path).map_err(|e| e.to_string())?;

    println!("Trace listing: {}", trace_path.display());
    println!("Metrics CSV: {}", csv_path.display());
    println!("  events={} cycles={} cycles/event={:.2}", metrics.events, metrics.cycles, metrics.cycles_per_event());
    println!("  hits={} misses={} allocations={} releases={}", metrics.hits, metrics.misses, metrics.allocations, metrics.releases);

    if std::env::var("KV_RTL_WAVE").is_ok() {
        sim.finalize_trace();
        println!(
            "Waveform written (inspect with gtkwave): {}",
            std::env::var("KV_RTL_WAVE").unwrap_or_default()
        );
    }

    println!("PASS: checkpoint 2 multi-page metadata tests");
    Ok(())
}

fn sync_reset(model: &mut MetadataTable, sim: &mut VerilatorSimulator) {
    model.reset();
    sim.reset();
}

fn compare_state(
    index: usize,
    event: Option<&AccessEvent>,
    model: &MetadataTable,
    sim: &VerilatorSimulator,
    model_result: EventResult,
) -> Result<(), String> {
    if sim.resp_valid() != model_result.resp_valid {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_valid"));
    }
    if sim.resp_error() != model_result.resp_error {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_error"));
    }
    if sim.resp_hit() != model_result.resp_hit {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_hit"));
    }
    if sim.resp_miss() != model_result.resp_miss {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_miss"));
    }
    if sim.resp_entry_id() != model_result.resp_entry_id {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_entry_id"));
    }
    if sim.resp_metadata() != model_result.resp_metadata {
        return Err(format_mismatch(index, event, model, sim, model_result, "resp_metadata"));
    }
    if sim.table_entries() != model.entries {
        return Err(format_mismatch(index, event, model, sim, model_result, "table_entries"));
    }
    if sim.hw_timestamp() != model.hw_timestamp {
        return Err(format_mismatch(index, event, model, sim, model_result, "hw_timestamp"));
    }
    Ok(())
}

fn format_mismatch(
    index: usize,
    event: Option<&AccessEvent>,
    model: &MetadataTable,
    sim: &VerilatorSimulator,
    model_result: EventResult,
    reason: &str,
) -> String {
    format!(
        "event_index={index}\nreason={reason}\nevent={event:?}\n\
         expected_table={:?}\nactual_table={:?}\n\
         expected_result={model_result:?}\n\
         sim_hit={} sim_miss={} sim_error={}",
        model.entries,
        sim.table_entries(),
        sim.resp_hit(),
        sim.resp_miss(),
        sim.resp_error(),
    )
}

fn apply_and_compare(
    index: usize,
    model: &mut MetadataTable,
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

fn run_table_edge_tests(sim: &mut VerilatorSimulator) -> Result<(), String> {
    let mut model = MetadataTable::new();
    sync_reset(&mut model, sim);

    // Allocate multiple pages: A 10, A 11, A 12
    for (i, page_id) in [10u16, 11, 12].iter().enumerate() {
        apply_and_compare(
            i,
            &mut model,
            sim,
            AccessEvent {
                request_id: 1,
                page_id: *page_id,
                timestamp: i as u16,
                operation: Operation::Allocate,
            },
        )?;
    }

    // Access hit: C 10, C 11
    apply_and_compare(
        3,
        &mut model,
        sim,
        AccessEvent {
            request_id: 1,
            page_id: 10,
            timestamp: 20,
            operation: Operation::Access,
        },
    )?;
    apply_and_compare(
        4,
        &mut model,
        sim,
        AccessEvent {
            request_id: 1,
            page_id: 11,
            timestamp: 21,
            operation: Operation::Access,
        },
    )?;

    // Miss: access untracked page
    let miss = AccessEvent {
        request_id: 1,
        page_id: 99,
        timestamp: 22,
        operation: Operation::Access,
    };
    apply_and_compare(5, &mut model, sim, miss)?;

    // Allocate when full (fill remaining slots)
    let mut idx = 6;
    for page_id in 100u16..200 {
        if model.entries.iter().filter(|e| e.valid).count() >= NUM_PAGES {
            break;
        }
        apply_and_compare(
            idx,
            &mut model,
            sim,
            AccessEvent {
                request_id: 1,
                page_id,
                timestamp: idx as u16,
                operation: Operation::Allocate,
            },
        )?;
        idx += 1;
    }

    // Table full -> alloc fail
    apply_and_compare(
        idx,
        &mut model,
        sim,
        AccessEvent {
            request_id: 1,
            page_id: 250,
            timestamp: 1,
            operation: Operation::Allocate,
        },
    )?;

    // Release
    apply_and_compare(
        idx + 1,
        &mut model,
        sim,
        AccessEvent {
            request_id: 1,
            page_id: 12,
            timestamp: 30,
            operation: Operation::Release,
        },
    )?;

    Ok(())
}

fn run_random_differential_tests(
    sim: &mut VerilatorSimulator,
    count: usize,
    seed: u64,
) -> Result<(), String> {
    let mut model = MetadataTable::new();
    sync_reset(&mut model, sim);

    let trace = generate_trace(&TraceConfig {
        seed,
        num_events: count,
        pattern: TracePattern::Random,
        ..TraceConfig::default()
    });

    for (index, event) in trace.iter().enumerate() {
        apply_and_compare(index, &mut model, sim, *event)?;
    }
    Ok(())
}

fn run_workload_with_metrics(
    sim: &mut VerilatorSimulator,
    pattern: TracePattern,
) -> Result<Metrics, String> {
    let mut model = MetadataTable::new();
    let mut metrics = Metrics::default();
    sync_reset(&mut model, sim);

    let trace = generate_trace(&TraceConfig {
        seed: 0xBEE_F00D,
        num_events: 200,
        pattern,
        ..TraceConfig::default()
    });

    for (index, event) in trace.iter().enumerate() {
        let start = sim.cycle_count();
        apply_and_compare(index, &mut model, sim, *event)?;
        metrics.record_cycles(sim.cycle_count() - start);
        metrics.record_event(
            event.operation,
            sim.resp_hit(),
            sim.resp_miss(),
            sim.resp_error(),
        );
        tick_idle(sim);
        model.step(None);
        metrics.record_cycles(1);
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential_random_smoke() {
        let mut sim = VerilatorSimulator::new();
        run_random_differential_tests(&mut sim, 100, 1).expect("random diff failed");
    }

    #[test]
    fn table_edge_cases() {
        let mut sim = VerilatorSimulator::new();
        run_table_edge_tests(&mut sim).expect("edge cases failed");
    }

    #[test]
    fn trace_generator_produces_events() {
        let trace = generate_trace(&TraceConfig {
            num_events: 20,
            ..TraceConfig::default()
        });
        assert_eq!(trace.len(), 20);
    }
}
