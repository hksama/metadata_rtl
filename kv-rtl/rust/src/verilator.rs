use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint, c_void};

use crate::Simulator;

#[repr(C)]
struct SimHandle(c_void);

extern "C" {
    fn sim_create() -> *mut SimHandle;
    fn sim_create_traced(path: *const c_char) -> *mut SimHandle;
    fn sim_destroy(sim: *mut SimHandle);
    fn sim_reset(sim: *mut SimHandle);
    fn sim_tick(sim: *mut SimHandle);
    fn sim_drive(sim: *mut SimHandle, input_valid: c_int, input_data: c_uint);
    fn sim_output_valid(sim: *const SimHandle) -> c_int;
    fn sim_output_data(sim: *const SimHandle) -> c_uint;
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

impl Simulator for VerilatorSimulator {
    fn reset(&mut self) {
        unsafe { sim_reset(self.handle) };
    }

    fn tick(&mut self) {
        unsafe { sim_tick(self.handle) };
    }

    fn drive_input(&mut self, valid: bool, data: u32) {
        unsafe {
            sim_drive(self.handle, i32::from(valid), data);
        }
    }

    fn output_valid(&self) -> bool {
        unsafe { sim_output_valid(self.handle) != 0 }
    }

    fn output_data(&self) -> u32 {
        unsafe { sim_output_data(self.handle) }
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
