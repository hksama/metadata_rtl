use crate::policy::{
    policy_score, policy_update_aging, refresh_scores, should_age_tick, PolicyConfig,
};
use crate::types::{
    sat_dec_u8, sat_inc_u16, sat_inc_u8, AccessEvent, Operation, PageMetadata, PageState,
    NUM_PAGES,
};

/// Golden software reference for the multi-page metadata table + hotness engine.
///
/// Mirrors the 2-cycle RTL pipeline:
///   step(Some(event))  → latch (cycle N)
///   step(None)         → commit  (cycle N+1), returns EventResult
///
/// Also models hw_timestamp increment and global aging ticks on idle cycles.
#[derive(Debug, Clone)]
pub struct MetadataTable {
    pub entries: Vec<PageMetadata>,
    pub hw_timestamp: u16,
    pub policy_config: PolicyConfig,
    pending_event: Option<AccessEvent>,
}

/// Response bundle after a committed event (matches RTL resp_* outputs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResult {
    pub resp_valid: bool,
    pub resp_error: bool,
    pub resp_hit: bool,
    pub resp_miss: bool,
    pub resp_entry_id: Option<u8>,
    pub resp_metadata: PageMetadata,
    pub resp_hotness_score: u8,
}

impl EventResult {
    /// Default idle/no-response state.
    pub const IDLE: Self = Self {
        resp_valid: false,
        resp_error: false,
        resp_hit: false,
        resp_miss: false,
        resp_entry_id: None,
        resp_metadata: PageMetadata::INVALID,
        resp_hotness_score: 0,
    };
}

impl MetadataTable {
    /// New table with default FREQUENCY policy.
    pub fn new() -> Self {
        Self::with_policy(PolicyConfig::default())
    }

    /// New table with explicit hotness policy config.
    pub fn with_policy(policy_config: PolicyConfig) -> Self {
        Self {
            entries: vec![PageMetadata::INVALID; NUM_PAGES],
            hw_timestamp: 0,
            policy_config,
            pending_event: None,
        }
    }

    /// Clear all entries and pending state; keep policy config.
    pub fn reset(&mut self) {
        let config = self.policy_config;
        *self = Self::with_policy(config);
    }

    /// Linear scan: index of valid entry matching page_id, or None.
    pub fn lookup_page(&self, page_id: u16) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.valid && e.page_id == page_id)
    }

    /// Lowest-index free slot (!valid), matching RTL alloc policy.
    pub fn find_free_entry(&self) -> Option<usize> {
        self.entries.iter().position(|e| !e.valid)
    }

    /// Advance one simulation cycle. Handles timestamp, aging ticks,
    /// pending-event commit, and new event latch. Returns result only on commit.
    pub fn step(&mut self, event: Option<AccessEvent>) -> EventResult {
        self.hw_timestamp = self.hw_timestamp.wrapping_add(1);

        if should_age_tick(
            self.policy_config.policy,
            self.hw_timestamp,
            self.policy_config.aging_interval,
        ) && self.pending_event.is_none()
        {
            self.apply_global_aging();
        }

        if let Some(pending) = self.pending_event.take() {
            return self.apply_committed_event(pending);
        }

        if let Some(ev) = event {
            self.pending_event = Some(ev);
        }

        EventResult::IDLE
    }

    /// Convenience: latch + commit in two steps (what tests usually call).
    pub fn submit_event(&mut self, event: AccessEvent) -> EventResult {
        let _ = self.step(Some(event));
        self.step(None)
    }

    /// Global aging: halve all aging counters, then refresh hotness scores.
    fn apply_global_aging(&mut self) {
        for entry in self.entries.iter_mut().filter(|e| e.valid) {
            entry.aging_counter = policy_update_aging(
                self.policy_config.policy,
                entry.aging_counter,
                false,
                true,
            );
        }
        refresh_scores(
            &mut self.entries,
            &self.policy_config,
            self.hw_timestamp,
        );
    }

    /// Commit a latched event; may trigger aging tick before handling op.
    fn apply_committed_event(&mut self, event: AccessEvent) -> EventResult {
        if should_age_tick(
            self.policy_config.policy,
            self.hw_timestamp,
            self.policy_config.aging_interval,
        ) {
            self.apply_global_aging();
        }

        match event.operation {
            Operation::Allocate => self.apply_allocate(event),
            Operation::Read | Operation::Write | Operation::Access => self.apply_access(event),
            Operation::Release => self.apply_release(event),
        }
    }

    /// ALLOCATE: reject duplicate or full table; else init entry with refcount=1.
    fn apply_allocate(&mut self, event: AccessEvent) -> EventResult {
        if self.lookup_page(event.page_id).is_some() {
            return EventResult {
                resp_valid: true,
                resp_error: true,
                ..EventResult::IDLE
            };
        }
        let Some(idx) = self.find_free_entry() else {
            return EventResult {
                resp_valid: true,
                resp_error: true,
                ..EventResult::IDLE
            };
        };
        let meta = PageMetadata {
            page_id: event.page_id,
            request_id: event.request_id,
            access_count: 0,
            last_access: 0,
            refcount: sat_inc_u8(0),
            valid: true,
            state: PageState::Valid,
            aging_counter: 0,
            hotness_score: 0,
        };
        self.entries[idx] = meta;
        let score = policy_score(
            self.policy_config.policy,
            self.policy_config.score_width,
            &meta,
            self.hw_timestamp,
        );
        self.entries[idx].hotness_score = score;
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: false,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: self.entries[idx],
            resp_hotness_score: score,
        }
    }

    /// ACCESS/READ/WRITE: hit updates count, last_access, aging, hotness;
    /// miss returns resp_miss without changing table.
    fn apply_access(&mut self, event: AccessEvent) -> EventResult {
        let Some(idx) = self.lookup_page(event.page_id) else {
            return EventResult {
                resp_valid: true,
                resp_error: false,
                resp_hit: false,
                resp_miss: true,
                resp_entry_id: None,
                resp_metadata: PageMetadata::INVALID,
                resp_hotness_score: 0,
            };
        };
        let entry = &mut self.entries[idx];
        entry.access_count = sat_inc_u16(entry.access_count);
        entry.last_access = event.timestamp;
        entry.aging_counter = policy_update_aging(
            self.policy_config.policy,
            entry.aging_counter,
            true,
            false,
        );
        let score = policy_score(
            self.policy_config.policy,
            self.policy_config.score_width,
            entry,
            self.hw_timestamp,
        );
        entry.hotness_score = score;
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: true,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: *entry,
            resp_hotness_score: score,
        }
    }

    /// RELEASE: decrement refcount; error if page missing or refcount already 0.
    fn apply_release(&mut self, event: AccessEvent) -> EventResult {
        let Some(idx) = self.lookup_page(event.page_id) else {
            return EventResult {
                resp_valid: true,
                resp_error: true,
                ..EventResult::IDLE
            };
        };
        if self.entries[idx].refcount == 0 {
            return EventResult {
                resp_valid: true,
                resp_error: true,
                ..EventResult::IDLE
            };
        }
        let entry = &mut self.entries[idx];
        entry.refcount = sat_dec_u8(entry.refcount);
        let score = entry.hotness_score;
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: true,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: *entry,
            resp_hotness_score: score,
        }
    }
}

impl Default for MetadataTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Backward-compatible alias from checkpoint 1.
pub type PageTracker = MetadataTable;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HotnessPolicy;

    #[test]
    fn multi_page_allocate_and_access() {
        let mut table = MetadataTable::new();
        assert!(!table
            .submit_event(AccessEvent {
                request_id: 1,
                page_id: 10,
                timestamp: 0,
                operation: Operation::Allocate,
            })
            .resp_error);

        let hit = table.submit_event(AccessEvent {
            request_id: 1,
            page_id: 10,
            timestamp: 5,
            operation: Operation::Access,
        });
        assert!(hit.resp_hit);
        assert_eq!(hit.resp_metadata.access_count, 1);
        assert_eq!(hit.resp_hotness_score, 1);
    }

    #[test]
    fn aging2_increments_on_access() {
        let mut table = MetadataTable::with_policy(PolicyConfig {
            policy: HotnessPolicy::Aging2Bit,
            ..PolicyConfig::default()
        });
        table
            .submit_event(AccessEvent {
                request_id: 1,
                page_id: 1,
                timestamp: 0,
                operation: Operation::Allocate,
            })
            .resp_error;
        let r = table.submit_event(AccessEvent {
            request_id: 1,
            page_id: 1,
            timestamp: 1,
            operation: Operation::Access,
        });
        assert_eq!(r.resp_metadata.aging_counter, 1);
        assert_eq!(r.resp_hotness_score, 1);
    }
}
