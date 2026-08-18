use std::path::PathBuf;

use kv_rtl::{
    evaluate_all_policies, generate_trace, tick_idle, write_policy_eval_csv, write_trace_listing,
    AccessEvent, EventResult, HotnessPolicy, MetadataSimulator, MetadataTable, Metrics,
    Operation, PolicyConfig, TraceConfig, TracePattern, VerilatorSimulator, NUM_PAGES,
};

fn main() {
    if let Err(msg) = run() {
        eprintln!("FAIL: {msg}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let policy_config = PolicyConfig::default();
    let mut sim = if let Ok(wave_path) = std::env::var("KV_RTL_WAVE") {
        VerilatorSimulator::with_trace(&wave_path).map_err(|e| e.to_string())?
    } else {
        VerilatorSimulator::new()
    };

    run_table_edge_tests(&mut sim, &policy_config)?;
    run_hotness_policy_tests(&mut sim, &policy_config)?;
    run_random_differential_tests(&mut sim, 300, 0xC0FF_EE02, &policy_config)?;

    let metrics = run_workload_with_metrics(&mut sim, TracePattern::Mixed, &policy_config)?;
    let traces_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("traces/synthetic");

    let trace = generate_trace(&TraceConfig {
        seed: 0x5151_0001,
        num_events: 50,
        pattern: TracePattern::Mixed,
        ..TraceConfig::default()
    });
    let trace_path = traces_dir.join("checkpoint3_trace.txt");
    let csv_path = traces_dir.join("checkpoint3_metrics.csv");
    let eval_path = traces_dir.join("checkpoint3_policy_eval.csv");

    write_trace_listing(&trace_path, &trace).map_err(|e| e.to_string())?;
    metrics.write_csv(&csv_path).map_err(|e| e.to_string())?;

    let eval_results = evaluate_all_policies(&policy_config);
    write_policy_eval_csv(&eval_results, &eval_path).map_err(|e| e.to_string())?;

    println!("Trace listing: {}", trace_path.display());
    println!("Metrics CSV: {}", csv_path.display());
    println!("Policy eval CSV: {}", eval_path.display());
    println!(
        "  events={} cycles={} cycles/event={:.2}",
        metrics.events,
        metrics.cycles,
        metrics.cycles_per_event()
    );
    println!(
        "  hits={} misses={} allocations={} releases={}",
        metrics.hits, metrics.misses, metrics.allocations, metrics.releases
    );
    println!("  policy_eval_runs={}", eval_results.len());

    if std::env::var("KV_RTL_WAVE").is_ok() {
        sim.finalize_trace();
        println!(
            "Waveform written (inspect with gtkwave): {}",
            std::env::var("KV_RTL_WAVE").unwrap_or_default()
        );
    }

    println!("PASS: checkpoint 3 hotness policy tests");
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
    if sim.resp_hotness_score() != model_result.resp_hotness_score {
        return Err(format_mismatch(
            index,
            event,
            model,
            sim,
            model_result,
            "resp_hotness_score",
        ));
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
         sim_hit={} sim_miss={} sim_error={} sim_score={}",
        model.entries,
        sim.table_entries(),
        sim.resp_hit(),
        sim.resp_miss(),
        sim.resp_error(),
        sim.resp_hotness_score(),
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

fn run_table_edge_tests(
    sim: &mut VerilatorSimulator,
    config: &PolicyConfig,
) -> Result<(), String> {
    let mut model = MetadataTable::with_policy(*config);
    sync_reset(&mut model, sim);

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

    let miss = AccessEvent {
        request_id: 1,
        page_id: 99,
        timestamp: 22,
        operation: Operation::Access,
    };
    apply_and_compare(5, &mut model, sim, miss)?;

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

fn run_hotness_policy_tests(
    sim: &mut VerilatorSimulator,
    _config: &PolicyConfig,
) -> Result<(), String> {
    // Frequency policy: repeated accesses increase hotness score.
    let freq_config = PolicyConfig {
        policy: HotnessPolicy::Frequency,
        ..PolicyConfig::default()
    };
    let mut model = MetadataTable::with_policy(freq_config);
    sync_reset(&mut model, sim);

    apply_and_compare(
        0,
        &mut model,
        sim,
        AccessEvent {
            request_id: 1,
            page_id: 42,
            timestamp: 0,
            operation: Operation::Allocate,
        },
    )?;

    for i in 1..=5 {
        apply_and_compare(
            i,
            &mut model,
            sim,
            AccessEvent {
                request_id: 1,
                page_id: 42,
                timestamp: i as u16,
                operation: Operation::Access,
            },
        )?;
    }

    if model.entries.iter().find(|e| e.page_id == 42).unwrap().hotness_score < 5 {
        return Err("frequency policy: score should reflect access count".into());
    }

    Ok(())
}

fn run_random_differential_tests(
    sim: &mut VerilatorSimulator,
    count: usize,
    seed: u64,
    config: &PolicyConfig,
) -> Result<(), String> {
    let mut model = MetadataTable::with_policy(*config);
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
    config: &PolicyConfig,
) -> Result<Metrics, String> {
    let mut model = MetadataTable::with_policy(*config);
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
        run_random_differential_tests(&mut sim, 100, 1, &PolicyConfig::default())
            .expect("random diff failed");
    }

    #[test]
    fn table_edge_cases() {
        let mut sim = VerilatorSimulator::new();
        run_table_edge_tests(&mut sim, &PolicyConfig::default()).expect("edge cases failed");
    }

    #[test]
    fn trace_generator_produces_events() {
        let trace = generate_trace(&TraceConfig {
            num_events: 20,
            ..TraceConfig::default()
        });
        assert_eq!(trace.len(), 20);
    }

    #[test]
    fn all_workload_patterns_generate() {
        use kv_rtl::all_workload_patterns;
        for (pattern, _) in all_workload_patterns() {
            let trace = generate_trace(&TraceConfig {
                num_events: 30,
                pattern,
                ..TraceConfig::default()
            });
            assert!(!trace.is_empty());
        }
    }
}
