use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::types::Operation;

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

impl Metrics {
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

    pub fn cycles_per_event(&self) -> f64 {
        if self.events == 0 {
            0.0
        } else {
            self.cycles as f64 / self.events as f64
        }
    }

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
