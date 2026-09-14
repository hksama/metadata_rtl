//! Offline ground-truth oracle for hotness evaluation.
//!
//! Given a future access window, predicts which pages will be reused.
//! This is NOT part of RTL — only an evaluation reference.

use std::collections::HashMap;

use crate::types::AccessEvent;

/// Count how many times each page_id appears as an ACCESS in trace[position..position+lookahead).
/// Used as ground-truth "future reuse" signal for policy evaluation (not in RTL).
pub fn future_reuse_counts(
    trace: &[AccessEvent],
    position: usize,
    lookahead: usize,
) -> HashMap<u16, u32> {
    let mut counts = HashMap::new();
    let end = (position + lookahead).min(trace.len());
    for event in &trace[position..end] {
        if event.operation.is_access() {
            *counts.entry(event.page_id).or_insert(0) += 1;
        }
    }
    counts
}

/// Binary labels: page is "oracle-hot" if it will be accessed at least once in the lookahead window.
pub fn future_hot_labels(
    trace: &[AccessEvent],
    position: usize,
    lookahead: usize,
    candidate_pages: &[u16],
) -> Vec<(u16, bool)> {
    let reuse = future_reuse_counts(trace, position, lookahead);
    candidate_pages
        .iter()
        .map(|&page_id| (page_id, reuse.get(&page_id).copied().unwrap_or(0) > 0))
        .collect()
}

/// Continuous oracle scores: raw future access count per page (for rank correlation).
pub fn future_hotness_scores(
    trace: &[AccessEvent],
    position: usize,
    lookahead: usize,
    candidate_pages: &[u16],
) -> Vec<(u16, u32)> {
    let reuse = future_reuse_counts(trace, position, lookahead);
    candidate_pages
        .iter()
        .map(|&page_id| (page_id, reuse.get(&page_id).copied().unwrap_or(0)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Operation;

    fn access(page_id: u16, ts: u16) -> AccessEvent {
        AccessEvent {
            request_id: 1,
            page_id,
            timestamp: ts,
            operation: Operation::Access,
        }
    }

    #[test]
    fn counts_future_reuse() {
        let trace = vec![access(1, 0), access(2, 1), access(1, 2), access(3, 3)];
        let counts = future_reuse_counts(&trace, 0, 4);
        assert_eq!(counts.get(&1), Some(&2));
        assert_eq!(counts.get(&2), Some(&1));
    }
}
