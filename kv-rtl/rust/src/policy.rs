//! Hotness policy engine — software mirror of RTL `kv_hotness_engine` / `kv_pkg` functions.
//!
//! Integer scores only. Floating point is avoided because it is unsuitable for
//! synthesizable RTL (no portable FP units, area/latency overhead, non-deterministic
//! rounding). Fixed-width integers map directly to hardware comparators and adders.

use crate::types::{HotnessPolicy, PageMetadata};

#[derive(Debug, Clone, Copy)]
pub struct PolicyConfig {
    pub policy: HotnessPolicy,
    pub score_width: u32,
    pub aging_interval: u32,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            policy: HotnessPolicy::Frequency,
            score_width: 8,
            aging_interval: 64,
        }
    }
}

/// policy_update(access_event, metadata) — update aging counter on access or global age tick.
pub fn policy_update_aging(
    policy: HotnessPolicy,
    aging_in: u8,
    is_access: bool,
    do_age: bool,
) -> u8 {
    let mut next = aging_in;
    if do_age {
        next >>= 1;
    } else if is_access {
        next = match policy {
            HotnessPolicy::Aging2Bit => aging_in.saturating_add(1).min(3),
            HotnessPolicy::Aging4Bit => aging_in.saturating_add(1).min(15),
            _ => aging_in,
        };
    }
    next
}

/// policy_score(metadata) — compute integer hotness score.
pub fn policy_score(
    policy: HotnessPolicy,
    score_width: u32,
    meta: &PageMetadata,
    current_timestamp: u16,
) -> u8 {
    if !meta.valid {
        return 0;
    }

    let raw: u32 = match policy {
        HotnessPolicy::Frequency => meta.access_count as u32,
        HotnessPolicy::Aging2Bit => (meta.aging_counter & 0x3) as u32,
        HotnessPolicy::Aging4Bit => (meta.aging_counter & 0xF) as u32,
        HotnessPolicy::ApproxLru => {
            let age = current_timestamp.wrapping_sub(meta.last_access) as u32;
            let recency = (u16::MAX as u32).saturating_sub(age);
            recency
        }
    };

    truncate_score(raw, score_width)
}

fn truncate_score(raw: u32, score_width: u32) -> u8 {
    let mask = if score_width >= 32 {
        u32::MAX
    } else {
        (1u32 << score_width) - 1
    };
    (raw & mask) as u8
}

/// Apply global aging tick if the hw timestamp hits the configured interval.
pub fn should_age_tick(policy: HotnessPolicy, hw_timestamp: u16, aging_interval: u32) -> bool {
    if aging_interval == 0 {
        return false;
    }
    matches!(policy, HotnessPolicy::Aging2Bit | HotnessPolicy::Aging4Bit)
        && hw_timestamp != 0
        && (hw_timestamp as u32 % aging_interval) == 0
}

/// Refresh hotness scores for all valid entries after metadata update.
pub fn refresh_scores(
    entries: &mut [PageMetadata],
    config: &PolicyConfig,
    current_timestamp: u16,
) {
    for entry in entries.iter_mut().filter(|e| e.valid) {
        entry.hotness_score =
            policy_score(config.policy, config.score_width, entry, current_timestamp);
    }
}

/// Estimate per-page metadata storage bits for a policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct HardwareCost {
    pub metadata_bits_per_page: u32,
    pub logic_ops_per_access: u32,
    pub comparators: u32,
    pub adders: u32,
    pub subtractors: u32,
    pub state_bits: u32,
}

pub fn estimate_hardware_cost(config: &PolicyConfig) -> HardwareCost {
    let base_bits = 16 + 16 + 16 + 16 + 8 + 1 + 2; // page_id, req_id, count, last_access, refcount, valid, state
    let aging_bits = config.policy.aging_counter_bits();
    let score_bits = config.score_width;

    let (ops, adders, subs, comparators) = match config.policy {
        HotnessPolicy::Frequency => (1, 1, 0, 0),
        HotnessPolicy::Aging2Bit | HotnessPolicy::Aging4Bit => (3, 2, 0, 1),
        HotnessPolicy::ApproxLru => (2, 0, 1, 0),
    };

    HardwareCost {
        metadata_bits_per_page: base_bits + aging_bits,
        logic_ops_per_access: ops,
        comparators,
        adders,
        subtractors: subs,
        state_bits: base_bits + aging_bits + score_bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PageState;

    #[test]
    fn frequency_score_is_access_count() {
        let meta = PageMetadata {
            page_id: 1,
            request_id: 1,
            access_count: 100,
            last_access: 0,
            refcount: 1,
            valid: true,
            state: PageState::Valid,
            aging_counter: 0,
            hotness_score: 0,
        };
        assert_eq!(policy_score(HotnessPolicy::Frequency, 8, &meta, 0), 100);
    }

    #[test]
    fn aging2_saturates_at_3() {
        assert_eq!(policy_update_aging(HotnessPolicy::Aging2Bit, 3, true, false), 3);
        assert_eq!(policy_update_aging(HotnessPolicy::Aging2Bit, 2, true, false), 3);
    }

    #[test]
    fn aging_halves_on_tick() {
        assert_eq!(policy_update_aging(HotnessPolicy::Aging2Bit, 3, false, true), 1);
    }
}
