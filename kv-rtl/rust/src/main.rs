use kv_rtl::{push_and_wait, tick_idle, Simulator, VerilatorSimulator};

fn main() {
    if let Err(msg) = run() {
        eprintln!("FAIL: {msg}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let mut sim = if let Ok(wave_path) = std::env::var("KV_RTL_WAVE") {
        VerilatorSimulator::with_trace(&wave_path)?
    } else {
        VerilatorSimulator::new()
    };

    run_checkpoint0_tests(&mut sim)?;

    if std::env::var("KV_RTL_WAVE").is_ok() {
        sim.finalize_trace();
        println!(
            "Waveform written (inspect with gtkwave): {}",
            std::env::var("KV_RTL_WAVE").unwrap_or_default()
        );
    }

    Ok(())
}

fn run_checkpoint0_tests(sim: &mut dyn Simulator) -> Result<(), &'static str> {
    // Reset behavior
    if sim.output_valid() {
        return Err("output_valid asserted after reset");
    }

    // Latency + arithmetic
    let (valid, out, cycles) = push_and_wait(sim, 41);
    if !valid {
        return Err("output_valid not asserted after one-cycle latency");
    }
    if out != 42 {
        return Err("output_data != input + 1");
    }
    if cycles != 2 {
        return Err("unexpected cycle count for single transaction");
    }

    tick_idle(sim);
    if sim.output_valid() {
        return Err("output_valid stuck high (protocol violation)");
    }

    // Reset clears pipeline
    sim.drive_input(true, 7);
    sim.tick();
    sim.reset();
    if sim.output_valid() {
        return Err("output_valid asserted after mid-flight reset");
    }

    // Burst of inputs
    let inputs = [3u32, 100, 0xffff_fff0];
    let start_cycles = sim.cycle_count();
    for &value in &inputs {
        let (v, o, _) = push_and_wait(sim, value);
        if !v {
            return Err("missing output_valid in burst");
        }
        if o != value.wrapping_add(1) {
            return Err("incorrect burst output");
        }
        tick_idle(sim);
    }
    let burst_cycles = sim.cycle_count() - start_cycles;

    println!("PASS: Rust harness validated reset, latency, and output");
    println!("  total_cycles={}", sim.cycle_count());
    println!("  burst_cycles={burst_cycles}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_cycle_by_cycle() {
        run_checkpoint0_tests(&mut VerilatorSimulator::new()).expect("harness tests failed");
    }

    #[test]
    fn detects_bad_latency_model() {
        struct LateSimulator {
            inner: VerilatorSimulator,
            extra_delay: bool,
        }

        impl Simulator for LateSimulator {
            fn reset(&mut self) {
                self.inner.reset();
                self.extra_delay = false;
            }
            fn tick(&mut self) {
                self.inner.tick();
            }
            fn drive_input(&mut self, valid: bool, data: u32) {
                self.inner.drive_input(valid, data);
            }
            fn output_valid(&self) -> bool {
                if self.extra_delay {
                    false
                } else {
                    self.inner.output_valid()
                }
            }
            fn output_data(&self) -> u32 {
                self.inner.output_data()
            }
            fn cycle_count(&self) -> u64 {
                self.inner.cycle_count()
            }
            fn enable_trace(&mut self, _path: &str) -> Result<(), &'static str> {
                Ok(())
            }
            fn finalize_trace(&mut self) {}
        }

        let mut sim = LateSimulator {
            inner: VerilatorSimulator::new(),
            extra_delay: true,
        };
        sim.drive_input(true, 1);
        sim.tick();
        sim.drive_input(false, 0);
        sim.tick();
        assert!(
            !sim.output_valid(),
            "test harness should catch injected latency bug"
        );
    }
}
