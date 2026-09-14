use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::oracle::future_hotness_scores;
use crate::policy::{estimate_hardware_cost, PolicyConfig};
use crate::types::{AccessEvent, HotnessPolicy, Operation};

/// Basic simulation counters (events, cycles, hits, misses, allocs, releases, errors).
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub events: u64,
    pub cycles: u64,
    pub hits: u64,
    pub misses: u64,
    pub allocations: u64,
    pub releases: u64,
    pub errors: u64,
}

/// Hot/cold classification quality vs oracle labels.
#[derive(Debug, Default, Clone)]
pub struct ClassificationMetrics {
    pub hot_precision: f64,
    pub hot_recall: f64,
    pub cold_precision: f64,
    pub cold_recall: f64,
    pub tp_hot: u64,
    pub fp_hot: u64,
    pub fn_hot: u64,
    pub tn_cold: u64,
}

/// Rank correlation metrics vs oracle future reuse scores.
#[derive(Debug, Default, Clone)]
pub struct RankingMetrics {
    pub spearman: f64,
    pub kendall: f64,
    pub top_k_overlap: f64,
}

/// Full result row for one (policy, workload) experiment.
#[derive(Debug, Clone)]
pub struct PolicyEvalResult {
    pub policy: HotnessPolicy,
    pub workload: String,
    pub classification: ClassificationMetrics,
    pub ranking: RankingMetrics,
    pub hardware_cost: crate::policy::HardwareCost,
    pub base_metrics: Metrics,
}

impl Metrics {
    /// Record one committed event outcome.
    pub fn record_event(&mut self, op: Operation, hit: bool, miss: bool, error: bool) {
        self.events += 1;
        if hit {
            self.hits += 1;
        }
        if miss {
            self.misses += 1;
        }
        if error {
            self.errors += 1;
        }
        match op {
            Operation::Allocate => self.allocations += 1,
            Operation::Release => self.releases += 1,
            _ => {}
        }
    }

    pub fn record_cycles(&mut self, cycles: u64) {
        self.cycles += cycles;
    }

    /// Average cycles per event (typically ~3 in harness due to idle ticks).
    pub fn cycles_per_event(&self) -> f64 {
        if self.events == 0 {
            0.0
        } else {
            self.cycles as f64 / self.events as f64
        }
    }

    /// Write checkpoint metrics CSV (events, hits, misses, …).
    pub fn write_csv(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "metric,value")?;
        writeln!(file, "events,{}", self.events)?;
        writeln!(file, "cycles,{}", self.cycles)?;
        writeln!(file, "cycles_per_event,{:.4}", self.cycles_per_event())?;
        writeln!(file, "hits,{}", self.hits)?;
        writeln!(file, "misses,{}", self.misses)?;
        writeln!(file, "allocations,{}", self.allocations)?;
        writeln!(file, "releases,{}", self.releases)?;
        writeln!(file, "errors,{}", self.errors)?;
        Ok(())
    }
}

/// Compute TP/FP/FN/TN and precision/recall for predicted-hot vs oracle-hot pages.
pub fn classification_metrics(
    predicted_hot: &[u16],
    oracle_hot: &[u16],
    all_pages: &[u16],
) -> ClassificationMetrics {
    let pred_set: std::collections::HashSet<u16> = predicted_hot.iter().copied().collect();
    let oracle_set: std::collections::HashSet<u16> = oracle_hot.iter().copied().collect();

    let mut tp = 0u64;
    let mut fp = 0u64;
    let mut fn_ = 0u64;
    let mut tn = 0u64;

    for &page in all_pages {
        let pred = pred_set.contains(&page);
        let truth = oracle_set.contains(&page);
        match (pred, truth) {
            (true, true) => tp += 1,
            (true, false) => fp += 1,
            (false, true) => fn_ += 1,
            (false, false) => tn += 1,
        }
    }

    ClassificationMetrics {
        hot_precision: safe_div(tp, tp + fp),
        hot_recall: safe_div(tp, tp + fn_),
        cold_precision: safe_div(tn, tn + fn_),
        cold_recall: safe_div(tn, tn + fp),
        tp_hot: tp,
        fp_hot: fp,
        fn_hot: fn_,
        tn_cold: tn,
    }
}

/// Spearman rank correlation coefficient (-1..1) between predicted and oracle score vectors.
pub fn spearman_correlation(predicted: &[u32], oracle: &[u32]) -> f64 {
    if predicted.len() != oracle.len() || predicted.len() < 2 {
        return 0.0;
    }
    let n = predicted.len() as f64;
    let rank_pred = ranks(predicted);
    let rank_oracle = ranks(oracle);
    let mut sum_d2 = 0.0;
    for i in 0..predicted.len() {
        let d = rank_pred[i] - rank_oracle[i];
        sum_d2 += d * d;
    }
    1.0 - (6.0 * sum_d2) / (n * (n * n - 1.0))
}

/// Simplified Kendall tau: fraction of concordant minus discordant page pairs.
pub fn kendall_correlation(predicted: &[u32], oracle: &[u32]) -> f64 {
    if predicted.len() != oracle.len() || predicted.len() < 2 {
        return 0.0;
    }
    let mut concordant = 0i64;
    let mut discordant = 0i64;
    let n = predicted.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let pred_order = (predicted[i] as i64) - (predicted[j] as i64);
            let oracle_order = (oracle[i] as i64) - (oracle[j] as i64);
            if pred_order.signum() == oracle_order.signum() {
                concordant += 1;
            } else if pred_order != 0 && oracle_order != 0 {
                discordant += 1;
            }
        }
    }
    let total = concordant + discordant;
    if total == 0 {
        0.0
    } else {
        (concordant - discordant) as f64 / total as f64
    }
}

/// |predicted top-K ∩ oracle top-K| / K — overlap of hottest page sets.
pub fn top_k_overlap(predicted: &[(u16, u32)], oracle: &[(u16, u32)], k: usize) -> f64 {
    let mut pred_sorted = predicted.to_vec();
    pred_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let mut oracle_sorted = oracle.to_vec();
    oracle_sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let pred_top: std::collections::HashSet<u16> =
        pred_sorted.iter().take(k).map(|(p, _)| *p).collect();
    let oracle_top: std::collections::HashSet<u16> =
        oracle_sorted.iter().take(k).map(|(p, _)| *p).collect();

    let overlap = pred_top.intersection(&oracle_top).count();
    if k == 0 {
        0.0
    } else {
        overlap as f64 / k as f64
    }
}

/// Assign average ranks with tie handling (helper for Spearman).
fn ranks(values: &[u32]) -> Vec<f64> {
    let mut indexed: Vec<(usize, u32)> = values.iter().copied().enumerate().collect();
    indexed.sort_by_key(|&(_, v)| v);
    let mut result = vec![0.0; values.len()];
    let mut i = 0;
    while i < indexed.len() {
        let mut j = i;
        while j + 1 < indexed.len() && indexed[j + 1].1 == indexed[i].1 {
            j += 1;
        }
        let avg_rank = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            result[indexed[k].0] = avg_rank;
        }
        i = j + 1;
    }
    result
}

fn safe_div(num: u64, den: u64) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 / den as f64
    }
}

/// Ranking metrics at a single trace position using oracle future_hotness_scores.
pub fn evaluate_at_position(
    trace: &[AccessEvent],
    position: usize,
    lookahead: usize,
    predicted_scores: &[(u16, u32)],
    k: usize,
) -> RankingMetrics {
    let pages: Vec<u16> = predicted_scores.iter().map(|(p, _)| *p).collect();
    let oracle = future_hotness_scores(trace, position, lookahead, &pages);
    let pred_vals: Vec<u32> = predicted_scores.iter().map(|(_, s)| *s).collect();
    let oracle_vals: Vec<u32> = oracle.iter().map(|(_, s)| *s).collect();

    RankingMetrics {
        spearman: spearman_correlation(&pred_vals, &oracle_vals),
        kendall: kendall_correlation(&pred_vals, &oracle_vals),
        top_k_overlap: top_k_overlap(predicted_scores, &oracle, k),
    }
}

/// Export policy evaluation sweep results to CSV.
pub fn write_policy_eval_csv(results: &[PolicyEvalResult], path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "policy,workload,hot_precision,hot_recall,cold_precision,cold_recall,\
         spearman,kendall,top_k_overlap,metadata_bits,logic_ops,state_bits"
    )?;
    for r in results {
        writeln!(
            file,
            "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{}",
            r.policy.name(),
            r.workload,
            r.classification.hot_precision,
            r.classification.hot_recall,
            r.classification.cold_precision,
            r.classification.cold_recall,
            r.ranking.spearman,
            r.ranking.kendall,
            r.ranking.top_k_overlap,
            r.hardware_cost.metadata_bits_per_page,
            r.hardware_cost.logic_ops_per_access,
            r.hardware_cost.state_bits,
        )?;
    }
    Ok(())
}

/// Assemble a PolicyEvalResult with hardware cost estimate attached.
pub fn build_policy_eval_result(
    policy: HotnessPolicy,
    workload: &str,
    config: &PolicyConfig,
    classification: ClassificationMetrics,
    ranking: RankingMetrics,
    base_metrics: Metrics,
) -> PolicyEvalResult {
    PolicyEvalResult {
        policy,
        workload: workload.to_string(),
        classification,
        ranking,
        hardware_cost: estimate_hardware_cost(config),
        base_metrics,
    }
}
