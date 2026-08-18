use crate::types::{
    sat_dec_u8, sat_inc_u16, sat_inc_u8, AccessEvent, Operation, PageMetadata, PageState,
    NUM_PAGES,
};

/// Golden software reference for multi-page metadata table updates.
#[derive(Debug, Clone)]
pub struct MetadataTable {
    pub entries: Vec<PageMetadata>,
    pub hw_timestamp: u16,
    pending_event: Option<AccessEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResult {
    pub resp_valid: bool,
    pub resp_error: bool,
    pub resp_hit: bool,
    pub resp_miss: bool,
    pub resp_entry_id: Option<u8>,
    pub resp_metadata: PageMetadata,
}

impl EventResult {
    pub const IDLE: Self = Self {
        resp_valid: false,
        resp_error: false,
        resp_hit: false,
        resp_miss: false,
        resp_entry_id: None,
        resp_metadata: PageMetadata::INVALID,
    };
}

impl MetadataTable {
    pub fn new() -> Self {
        Self {
            entries: vec![PageMetadata::INVALID; NUM_PAGES],
            hw_timestamp: 0,
            pending_event: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn lookup_page(&self, page_id: u16) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.valid && e.page_id == page_id)
    }

    pub fn find_free_entry(&self) -> Option<usize> {
        self.entries.iter().position(|e| !e.valid)
    }

    pub fn step(&mut self, event: Option<AccessEvent>) -> EventResult {
        self.hw_timestamp = self.hw_timestamp.wrapping_add(1);

        if let Some(pending) = self.pending_event.take() {
            return self.apply_committed_event(pending);
        }

        if let Some(ev) = event {
            self.pending_event = Some(ev);
        }

        EventResult::IDLE
    }

    pub fn submit_event(&mut self, event: AccessEvent) -> EventResult {
        let _ = self.step(Some(event));
        self.step(None)
    }

    fn apply_committed_event(&mut self, event: AccessEvent) -> EventResult {
        match event.operation {
            Operation::Allocate => self.apply_allocate(event),
            Operation::Read | Operation::Write | Operation::Access => self.apply_access(event),
            Operation::Release => self.apply_release(event),
        }
    }

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
        };
        self.entries[idx] = meta;
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: false,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: meta,
        }
    }

    fn apply_access(&mut self, event: AccessEvent) -> EventResult {
        let Some(idx) = self.lookup_page(event.page_id) else {
            return EventResult {
                resp_valid: true,
                resp_error: false,
                resp_hit: false,
                resp_miss: true,
                resp_entry_id: None,
                resp_metadata: PageMetadata::INVALID,
            };
        };
        let entry = &mut self.entries[idx];
        entry.access_count = sat_inc_u16(entry.access_count);
        entry.last_access = event.timestamp;
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: true,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: *entry,
        }
    }

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
        EventResult {
            resp_valid: true,
            resp_error: false,
            resp_hit: true,
            resp_miss: false,
            resp_entry_id: Some(idx as u8),
            resp_metadata: *entry,
        }
    }
}

impl Default for MetadataTable {
    fn default() -> Self {
        Self::new()
    }
}

// Backward-compatible alias used by earlier checkpoints.
pub type PageTracker = MetadataTable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_page_allocate_and_access() {
        let mut table = MetadataTable::new();
        assert!(!table.submit_event(AccessEvent {
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
    }
}
