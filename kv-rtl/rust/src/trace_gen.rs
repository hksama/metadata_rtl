use crate::types::{AccessEvent, Operation, NUM_PAGES};

/// Workload pattern for synthetic trace generation.
#[derive(Debug, Clone, Copy)]
pub enum TracePattern {
    Mixed,
    HotSpot,
    Sequential,
    Random,
    Bursty,
}

#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub seed: u64,
    pub num_events: usize,
    pub pattern: TracePattern,
    pub request_id: u16,
    pub page_pool: u16,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            seed: 0xACE5_EED,
            num_events: 200,
            pattern: TracePattern::Mixed,
            request_id: 1,
            page_pool: 32,
        }
    }
}

struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (self.state >> 16) as u32
    }

    fn next_u16(&mut self) -> u16 {
        (self.next_u32() & 0xFFFF) as u16
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u32() as usize) % bound
        }
    }
}

/// Generate a synthetic access trace with allocations, accesses, releases, and misses.
pub fn generate_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let mut rng = LcgRng::new(config.seed);
    let mut trace = Vec::with_capacity(config.num_events);
    let mut allocated: Vec<u16> = Vec::new();
    let mut ts = 0u16;

  let hot_pages = [
        (config.page_pool / 4).max(1),
        (config.page_pool / 4 + 1).max(2),
    ];
    let warm_pages = [
        (config.page_pool / 2).max(3),
        (config.page_pool / 2 + 1).max(4),
    ];
    let cold_base = config.page_pool.saturating_sub(4).max(5);

    for i in 0..config.num_events {
        ts = ts.wrapping_add(1);
        let op_roll = rng.next_u32() % 100;
        let event = match config.pattern {
            TracePattern::Sequential => {
                if allocated.len() < 4 && op_roll < 40 {
                    let page_id = (i as u16) + 1;
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Allocate,
                    }
                } else if !allocated.is_empty() {
                    let idx = i % allocated.len();
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: allocated[idx],
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                } else {
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: (i as u16) + 100,
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                }
            }
            TracePattern::HotSpot => {
                if allocated.len() < NUM_PAGES && op_roll < 25 {
                    let page_id = hot_pages[rng.next_usize(hot_pages.len())];
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Allocate,
                    }
                } else if op_roll < 85 && !allocated.is_empty() {
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: hot_pages[rng.next_usize(hot_pages.len())],
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                } else if !allocated.is_empty() && op_roll < 95 {
                    let page_id = allocated[rng.next_usize(allocated.len())];
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Release,
                    }
                } else {
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: cold_base + rng.next_u16() % 8,
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                }
            }
            TracePattern::Random => pick_random_event(
                &mut rng,
                config,
                &mut allocated,
                ts,
                op_roll,
                cold_base,
            ),
            TracePattern::Bursty => {
                let in_burst = (i / 8) % 2 == 0;
                if in_burst {
                    if allocated.len() < NUM_PAGES && op_roll < 50 {
                        let page_id = config.page_pool + (i as u16 % 8);
                        AccessEvent {
                            request_id: config.request_id,
                            page_id,
                            timestamp: ts,
                            operation: Operation::Allocate,
                        }
                    } else if !allocated.is_empty() {
                        AccessEvent {
                            request_id: config.request_id,
                            page_id: allocated[rng.next_usize(allocated.len())],
                            timestamp: ts,
                            operation: Operation::Access,
                        }
                    } else {
                        AccessEvent {
                            request_id: config.request_id,
                            page_id: 1,
                            timestamp: ts,
                            operation: Operation::Access,
                        }
                    }
                } else {
                    pick_random_event(
                        &mut rng,
                        config,
                        &mut allocated,
                        ts,
                        op_roll,
                        cold_base,
                    )
                }
            }
            TracePattern::Mixed => {
                if allocated.len() < NUM_PAGES && op_roll < 20 {
                    let page_id = match rng.next_u32() % 3 {
                        0 => hot_pages[rng.next_usize(hot_pages.len())],
                        1 => warm_pages[rng.next_usize(warm_pages.len())],
                        _ => cold_base + rng.next_u16() % 16,
                    };
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Allocate,
                    }
                } else if op_roll < 75 {
                    let page_id = if rng.next_u32() % 100 < 60 && !allocated.is_empty() {
                        allocated[rng.next_usize(allocated.len())]
                    } else {
                        cold_base + rng.next_u16() % 32
                    };
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                } else if !allocated.is_empty() && op_roll < 90 {
                    let page_id = allocated[rng.next_usize(allocated.len())];
                    AccessEvent {
                        request_id: config.request_id,
                        page_id,
                        timestamp: ts,
                        operation: Operation::Release,
                    }
                } else {
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: cold_base + rng.next_u16() % 64,
                        timestamp: ts,
                        operation: Operation::Access,
                    }
                }
            }
        };

        update_allocated(&mut allocated, &event);
        trace.push(event);
    }

    trace
}

fn pick_random_event(
    rng: &mut LcgRng,
    config: &TraceConfig,
    allocated: &mut Vec<u16>,
    ts: u16,
    op_roll: u32,
    cold_base: u16,
) -> AccessEvent {
    if allocated.len() < NUM_PAGES && op_roll < 30 {
        AccessEvent {
            request_id: config.request_id,
            page_id: 1 + rng.next_u16() % config.page_pool.max(1),
            timestamp: ts,
            operation: Operation::Allocate,
        }
    } else if !allocated.is_empty() && op_roll < 80 {
        AccessEvent {
            request_id: config.request_id,
            page_id: allocated[rng.next_usize(allocated.len())],
            timestamp: ts,
            operation: Operation::Access,
        }
    } else if !allocated.is_empty() && op_roll < 90 {
        AccessEvent {
            request_id: config.request_id,
            page_id: allocated[rng.next_usize(allocated.len())],
            timestamp: ts,
            operation: Operation::Release,
        }
    } else {
        AccessEvent {
            request_id: config.request_id,
            page_id: cold_base + rng.next_u16() % 32,
            timestamp: ts,
            operation: Operation::Access,
        }
    }
}

fn update_allocated(allocated: &mut Vec<u16>, event: &AccessEvent) {
    match event.operation {
        Operation::Allocate => {
            if !allocated.contains(&event.page_id) {
                allocated.push(event.page_id);
            }
        }
        Operation::Release => {
            if let Some(pos) = allocated.iter().position(|p| *p == event.page_id) {
                allocated.remove(pos);
            }
        }
        _ => {}
    }
}

/// Write a human-readable trace listing (A/R prefixes) for inspection.
pub fn write_trace_listing(path: &std::path::Path, trace: &[AccessEvent]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    for event in trace {
        let prefix = match event.operation {
            Operation::Allocate => 'A',
            Operation::Release => 'R',
            _ => 'C',
        };
        writeln!(file, "{prefix} {}", event.page_id)?;
    }
    Ok(())
}
