//! Policy evaluation harness for checkpoint 3.

use crate::metrics::{
    build_policy_eval_result, classification_metrics, evaluate_at_position, Metrics,
    PolicyEvalResult, RankingMetrics,
};
use crate::model::MetadataTable;
use crate::oracle::future_hot_labels;
use crate::policy::PolicyConfig;
use crate::trace_gen::{all_workload_patterns, generate_trace, TraceConfig, TracePattern, AGING_INTERVALS, ZIPF_ALPHAS};
use crate::types::{AccessEvent, HotnessPolicy};

const LOOKAHEAD: usize = 32;
const TOP_K: usize = 4;

/// Run a trace through the software reference model and collect evaluation metrics.
pub fn evaluate_policy_on_trace(
    config: &PolicyConfig,
    _pattern: TracePattern,
    workload_name: &str,
    trace: &[AccessEvent],
) -> PolicyEvalResult {
    let mut model = MetadataTable::with_policy(*config);
    let mut metrics = Metrics::default();
    let mut all_predicted_hot = Vec::new();
    let mut all_oracle_hot = Vec::new();
    let mut all_pages = Vec::new();
    let mut ranking_accum = RankingMetrics::default();
    let mut ranking_samples = 0u64;

    for (i, event) in trace.iter().enumerate() {
        let start_cycles = model.hw_timestamp as u64;
        let result = model.submit_event(*event);
        metrics.record_cycles((model.hw_timestamp as u64).saturating_sub(start_cycles) + 2);
        metrics.record_event(
            event.operation,
            result.resp_hit,
            result.resp_miss,
            result.resp_error,
        );

        if event.operation.is_access() && i + LOOKAHEAD < trace.len() {
            let candidates: Vec<u16> = model
                .entries
                .iter()
                .filter(|e| e.valid)
                .map(|e| e.page_id)
                .collect();

            if candidates.is_empty() {
                continue;
            }

            let predicted_scores: Vec<(u16, u32)> = model
                .entries
                .iter()
                .filter(|e| e.valid)
                .map(|e| (e.page_id, e.hotness_score as u32))
                .collect();

            let median_score = median(predicted_scores.iter().map(|(_, s)| *s).collect());
            let pred_hot: Vec<u16> = predicted_scores
                .iter()
                .filter(|(_, s)| *s >= median_score)
                .map(|(p, _)| *p)
                .collect();

            let oracle_labels = future_hot_labels(trace, i, LOOKAHEAD, &candidates);
            let oracle_hot: Vec<u16> = oracle_labels
                .iter()
                .filter(|(_, hot)| *hot)
                .map(|(p, _)| *p)
                .collect();

            all_predicted_hot.extend(&pred_hot);
            all_oracle_hot.extend(&oracle_hot);
            all_pages.extend(&candidates);

            let rank = evaluate_at_position(trace, i, LOOKAHEAD, &predicted_scores, TOP_K);
            ranking_accum.spearman += rank.spearman;
            ranking_accum.kendall += rank.kendall;
            ranking_accum.top_k_overlap += rank.top_k_overlap;
            ranking_samples += 1;
        }
    }

    all_pages.sort_unstable();
    all_pages.dedup();

    let classification = classification_metrics(&all_predicted_hot, &all_oracle_hot, &all_pages);

    let ranking = if ranking_samples > 0 {
        RankingMetrics {
            spearman: ranking_accum.spearman / ranking_samples as f64,
            kendall: ranking_accum.kendall / ranking_samples as f64,
            top_k_overlap: ranking_accum.top_k_overlap / ranking_samples as f64,
        }
    } else {
        RankingMetrics::default()
    };

    build_policy_eval_result(
        config.policy,
        workload_name,
        config,
        classification,
        ranking,
        metrics,
    )
}

fn median(mut values: Vec<u32>) -> u32 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    values[values.len() / 2]
}

/// Evaluate all policies across all standard workloads.
pub fn evaluate_all_policies(base_config: &PolicyConfig) -> Vec<PolicyEvalResult> {
    let mut results = Vec::new();

    for policy in HotnessPolicy::all() {
        let mut config = *base_config;
        config.policy = policy;

        for (pattern, name) in all_workload_patterns() {
            let mut trace_config = TraceConfig {
                seed: 0xC3_0013,
                num_events: 150,
                pattern,
                ..TraceConfig::default()
            };

            if pattern == TracePattern::Zipf {
                for &alpha in &ZIPF_ALPHAS {
                    trace_config.zipf_alpha = alpha;
                    let trace = generate_trace(&trace_config);
                    let workload = format!("zipf_alpha_{alpha}");
                    results.push(evaluate_policy_on_trace(&config, pattern, &workload, &trace));
                }
            } else {
                let trace = generate_trace(&trace_config);
                results.push(evaluate_policy_on_trace(&config, pattern, name, &trace));
            }
        }
    }

    results
}

/// Sweep aging intervals for 2-bit and 4-bit policies.
pub fn evaluate_aging_sweep(base_config: &PolicyConfig) -> Vec<PolicyEvalResult> {
    let mut results = Vec::new();
    let pattern = TracePattern::Bursty;

    for policy in [HotnessPolicy::Aging2Bit, HotnessPolicy::Aging4Bit] {
        for &interval in &AGING_INTERVALS {
            let mut config = *base_config;
            config.policy = policy;
            config.aging_interval = interval;

            let trace = generate_trace(&TraceConfig {
                seed: 0xA61_6000,
                num_events: 200,
                pattern,
                ..TraceConfig::default()
            });

            let workload = format!("{}_interval_{}", policy.name(), interval);
            results.push(evaluate_policy_on_trace(&config, pattern, &workload, &trace));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_frequency_on_single_hot() {
        let config = PolicyConfig {
            policy: HotnessPolicy::Frequency,
            ..PolicyConfig::default()
        };
        let trace = generate_trace(&TraceConfig {
            pattern: TracePattern::SingleHot,
            num_events: 50,
            ..TraceConfig::default()
        });
        let result = evaluate_policy_on_trace(&config, TracePattern::SingleHot, "single_hot", &trace);
        assert!(result.base_metrics.events > 0);
    }
}
