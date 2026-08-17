use crate::types::{
    sat_dec_u8, sat_inc_u16, sat_inc_u8, AccessEvent, Operation, PageMetadata, PageState,
};

/// Golden software reference for single-page metadata updates.
#[derive(Debug, Clone)]
pub struct PageTracker {
    pub metadata: PageMetadata,
    pub hw_timestamp: u16,
    pending_event: Option<AccessEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventResult {
    pub resp_valid: bool,
    pub resp_error: bool,
}

impl PageTracker {
    pub fn new() -> Self {
        Self {
            metadata: PageMetadata::INVALID,
            hw_timestamp: 0,
            pending_event: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Advance one clock cycle, optionally presenting a new event for latching.
    pub fn step(&mut self, event: Option<AccessEvent>) -> EventResult {
        self.hw_timestamp = self.hw_timestamp.wrapping_add(1);

        if let Some(pending) = self.pending_event.take() {
            let error = self.apply_committed_event(pending);
            return EventResult {
                resp_valid: true,
                resp_error: error,
            };
        }

        if let Some(ev) = event {
            self.pending_event = Some(ev);
        }

        EventResult {
            resp_valid: false,
            resp_error: false,
        }
    }

    /// Two-cycle handshake matching RTL: latch then commit.
    pub fn submit_event(&mut self, event: AccessEvent) -> EventResult {
        let _ = self.step(Some(event));
        self.step(None)
    }

    fn apply_committed_event(&mut self, event: AccessEvent) -> bool {
        match event.operation {
            Operation::Allocate => self.apply_allocate(event),
            Operation::Read | Operation::Write => self.apply_access(event),
            Operation::Release => self.apply_release(event),
        }
    }

    fn apply_allocate(&mut self, event: AccessEvent) -> bool {
        if self.metadata.valid {
            return true;
        }
        self.metadata = PageMetadata {
            page_id: event.page_id,
            request_id: event.request_id,
            access_count: 0,
            last_access: 0,
            refcount: sat_inc_u8(0),
            valid: true,
            state: PageState::Valid,
        };
        false
    }

    fn apply_access(&mut self, event: AccessEvent) -> bool {
        if !self.page_matches(event.page_id) {
            return true;
        }
        self.metadata.access_count = sat_inc_u16(self.metadata.access_count);
        self.metadata.last_access = event.timestamp;
        false
    }

    fn apply_release(&mut self, event: AccessEvent) -> bool {
        if !self.page_matches(event.page_id) {
            return true;
        }
        if self.metadata.refcount == 0 {
            return true;
        }
        self.metadata.refcount = sat_dec_u8(self.metadata.refcount);
        false
    }

    fn page_matches(&self, page_id: u16) -> bool {
        self.metadata.valid && self.metadata.page_id == page_id
    }

    #[cfg_attr(test, allow(dead_code))]
    pub fn set_metadata_for_test(&mut self, metadata: PageMetadata, hw_timestamp: u16) {
        self.metadata = metadata;
        self.hw_timestamp = hw_timestamp;
        self.pending_event = None;
    }
}

impl Default for PageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_read_release_flow() {
        let mut model = PageTracker::new();
        model.step(None);
        model.step(None);

        let alloc = AccessEvent {
            request_id: 1,
            page_id: 7,
            timestamp: 10,
            operation: Operation::Allocate,
        };
        assert!(!model.submit_event(alloc).resp_error);

        let read = AccessEvent {
            request_id: 1,
            page_id: 7,
            timestamp: 42,
            operation: Operation::Read,
        };
        assert!(!model.submit_event(read).resp_error);
        assert_eq!(model.metadata.access_count, 1);
        assert_eq!(model.metadata.last_access, 42);
    }
}
