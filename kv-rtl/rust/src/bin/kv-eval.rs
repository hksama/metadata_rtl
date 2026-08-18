//! Policy evaluation CLI for checkpoint 3 experiments.

use std::path::PathBuf;

use kv_rtl::{
    evaluate_aging_sweep, evaluate_all_policies, write_policy_eval_csv, PolicyConfig,
};

fn main() {
    if let Err(msg) = run() {
        eprintln!("FAIL: {msg}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let all_policies = args.iter().any(|a| a == "--all-policies");
    let aging_sweep = args.iter().any(|a| a == "--aging-sweep");

    let config = PolicyConfig::default();
    let traces_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("traces/synthetic");

    if aging_sweep {
        let results = evaluate_aging_sweep(&config);
        let path = traces_dir.join("checkpoint3_aging_sweep.csv");
        write_policy_eval_csv(&results, &path).map_err(|e| e.to_string())?;
        println!("Aging sweep: {} results -> {}", results.len(), path.display());
    }

    if all_policies || (!aging_sweep) {
        let results = evaluate_all_policies(&config);
        let path = traces_dir.join("checkpoint3_policy_eval.csv");
        write_policy_eval_csv(&results, &path).map_err(|e| e.to_string())?;
        println!(
            "Policy evaluation: {} results -> {}",
            results.len(),
            path.display()
        );
        for r in &results[..results.len().min(5)] {
            println!(
                "  {} / {}: spearman={:.3} hot_prec={:.3} metadata_bits={}",
                r.policy.name(),
                r.workload,
                r.ranking.spearman,
                r.classification.hot_precision,
                r.hardware_cost.metadata_bits_per_page
            );
        }
    }

    Ok(())
}
