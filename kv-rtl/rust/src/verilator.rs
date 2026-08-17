use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint, c_void};

use crate::types::{AccessEvent, PageMetadata, PageState};
use crate::MetadataSimulator;

#[repr(C)]
struct SimHandle(c_void);

#[repr(C)]
struct KvEvent {
    operation: u8,
    request_id: u16,
    page_id: u16,
    event_timestamp: u16,
}

#[repr(C)]
struct KvMetadata {
    valid: u8,
    page_id: u16,
    request_id: u16,
    access_count: u16,
    last_access: u16,
    refcount: u8,
    state: u8,
}

extern "C" {
    fn sim_create() -> *mut SimHandle;
    fn sim_create_traced(path: *const c_char) -> *mut SimHandle;
    fn sim_destroy(sim: *mut SimHandle);
    fn sim_reset(sim: *mut SimHandle);
    fn sim_tick(sim: *mut SimHandle);
    fn sim_drive_event(sim: *mut SimHandle, event_valid: c_int, event: *const KvEvent);
    fn sim_deassert_event(sim: *mut SimHandle);
    fn sim_resp_valid(sim: *const SimHandle) -> c_int;
    fn sim_resp_error(sim: *const SimHandle) -> c_int;
    fn sim_read_metadata(sim: *const SimHandle, out: *mut KvMetadata);
    fn sim_hw_timestamp(sim: *const SimHandle) -> c_uint;
    fn sim_cycle_count(sim: *const SimHandle) -> u64;
    fn sim_finalize_trace(sim: *mut SimHandle);
}

pub struct VerilatorSimulator {
    handle: *mut SimHandle,
}

impl VerilatorSimulator {
    pub fn new() -> Self {
        let handle = unsafe { sim_create() };
        assert!(!handle.is_null(), "sim_create failed");
        Self { handle }
    }

    pub fn with_trace(path: &str) -> Result<Self, &'static str> {
        let c_path = CString::new(path).map_err(|_| "trace path contains interior NUL")?;
        let handle = unsafe { sim_create_traced(c_path.as_ptr()) };
        if handle.is_null() {
            return Err("sim_create_traced failed");
        }
        Ok(Self { handle })
    }
}

impl Drop for VerilatorSimulator {
    fn drop(&mut self) {
        unsafe { sim_destroy(self.handle) };
    }
}

fn to_kv_event(event: &AccessEvent) -> KvEvent {
    KvEvent {
        operation: event.operation as u8,
        request_id: event.request_id,
        page_id: event.page_id,
        event_timestamp: event.timestamp,
    }
}

fn from_kv_metadata(raw: KvMetadata) -> PageMetadata {
    PageMetadata {
        page_id: raw.page_id,
        request_id: raw.request_id,
        access_count: raw.access_count,
        last_access: raw.last_access,
        refcount: raw.refcount,
        valid: raw.valid != 0,
        state: if raw.state == PageState::Valid as u8 {
            PageState::Valid
        } else {
            PageState::Invalid
        },
    }
}

impl MetadataSimulator for VerilatorSimulator {
    fn reset(&mut self) {
        unsafe { sim_reset(self.handle) };
    }

    fn tick(&mut self) {
        unsafe { sim_tick(self.handle) };
    }

    fn drive_event(&mut self, event: &AccessEvent) {
        let kv = to_kv_event(event);
        unsafe { sim_drive_event(self.handle, 1, &kv) };
    }

    fn deassert_event(&mut self) {
        unsafe { sim_deassert_event(self.handle) };
    }

    fn resp_valid(&self) -> bool {
        unsafe { sim_resp_valid(self.handle) != 0 }
    }

    fn resp_error(&self) -> bool {
        unsafe { sim_resp_error(self.handle) != 0 }
    }

    fn metadata(&self) -> PageMetadata {
        let mut raw = KvMetadata {
            valid: 0,
            page_id: 0,
            request_id: 0,
            access_count: 0,
            last_access: 0,
            refcount: 0,
            state: 0,
        };
        unsafe { sim_read_metadata(self.handle, &mut raw) };
        from_kv_metadata(raw)
    }

    fn hw_timestamp(&self) -> u16 {
        unsafe { sim_hw_timestamp(self.handle) as u16 }
    }

    fn cycle_count(&self) -> u64 {
        unsafe { sim_cycle_count(self.handle) }
    }

    fn enable_trace(&mut self, _path: &str) -> Result<(), &'static str> {
        Err("enable tracing via VerilatorSimulator::with_trace at construction time")
    }

    fn finalize_trace(&mut self) {
        unsafe { sim_finalize_trace(self.handle) };
    }
}

unsafe impl Send for VerilatorSimulator {}
