use crate::types::{AccessEvent, Operation, NUM_PAGES};

/// Workload pattern for synthetic trace generation (checkpoint 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracePattern {
    /// Mixed hot/warm/cold (checkpoint 2 compatibility).
    Mixed,
    /// 0 1 2 3 4 5 6 7 ...
    Sequential,
    /// Uniform random page IDs.
    UniformRandom,
    /// Single hot page: 1 1 1 1 1 1 1 4 7 3
    SingleHot,
    /// Multiple hot pages: 1 2 1 3 2 1 3 2 1 3
    MultipleHot,
    /// Bursty: A A A A A → B B B B B → C C C C C → A A A A A
    Bursty,
    /// Zipf-distributed page accesses.
    Zipf,
    /// Agent/LLM-inspired: shared prefix + per-request suffixes.
    Agentic,
    /// Checkpoint 2 alias.
    HotSpot,
    Random,
}

/// Parameters controlling trace generation (seed, length, pattern-specific knobs).
#[derive(Debug, Clone)]
pub struct TraceConfig {
    pub seed: u64,
    pub num_events: usize,
    pub pattern: TracePattern,
    pub request_id: u16,
    pub page_pool: u16,
    /// Zipf skew parameter (alpha).
    pub zipf_alpha: f64,
    /// Number of requests in agentic workload.
    pub num_requests: usize,
    /// Shared prefix length (KV page IDs).
    pub prefix_len: usize,
    /// Suffix length per request.
    pub suffix_len: usize,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            seed: 0xACE5_EED,
            num_events: 200,
            pattern: TracePattern::Mixed,
            request_id: 1,
            page_pool: 32,
            zipf_alpha: 1.0,
            num_requests: 4,
            prefix_len: 8,
            suffix_len: 6,
        }
    }
}

/// Simple deterministic LCG for reproducible synthetic traces (not crypto-grade).
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

    fn next_f64(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64)
    }
}

/// Dispatch to pattern-specific generator based on config.pattern.
pub fn generate_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    match config.pattern {
        TracePattern::Agentic => generate_agentic_trace(config),
        TracePattern::Sequential => generate_sequential_trace(config),
        TracePattern::UniformRandom | TracePattern::Random => {
            generate_uniform_random_trace(config)
        }
        TracePattern::SingleHot => generate_single_hot_trace(config),
        TracePattern::MultipleHot => generate_multiple_hot_trace(config),
        TracePattern::Bursty => generate_bursty_trace(config),
        TracePattern::Zipf => generate_zipf_trace(config),
        _ => generate_mixed_trace(config),
    }
}

/// Sequential scan: allocate pages 0,1,2… then cycle through them.
/// Tests whether frequency policy falsely marks scans as hot.
fn generate_sequential_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let mut trace = Vec::with_capacity(config.num_events);
    let mut ts = 0u16;
    let mut allocated = Vec::new();

    for i in 0..config.num_events {
        ts = ts.wrapping_add(1);
        let event = if allocated.len() < 8 && i % 3 == 0 {
            let page_id = i as u16;
            AccessEvent {
                request_id: config.request_id,
                page_id,
                timestamp: ts,
                operation: Operation::Allocate,
            }
        } else if !allocated.is_empty() {
            AccessEvent {
                request_id: config.request_id,
                page_id: (i % allocated.len()) as u16,
                timestamp: ts,
                operation: Operation::Access,
            }
        } else {
            AccessEvent {
                request_id: config.request_id,
                page_id: i as u16,
                timestamp: ts,
                operation: Operation::Access,
            }
        };
        update_allocated(&mut allocated, &event);
        trace.push(event);
    }
    trace
}

/// Uniform random page IDs — minimal locality baseline.
fn generate_uniform_random_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let mut rng = LcgRng::new(config.seed);
    let mut trace = Vec::with_capacity(config.num_events);
    let mut allocated = Vec::new();
    let mut ts = 0u16;
    let pool = config.page_pool.max(1);

    for _ in 0..config.num_events {
        ts = ts.wrapping_add(1);
        let event = if allocated.len() < NUM_PAGES && rng.next_u32() % 5 == 0 {
            AccessEvent {
                request_id: config.request_id,
                page_id: 1 + rng.next_u16() % pool,
                timestamp: ts,
                operation: Operation::Allocate,
            }
        } else {
            AccessEvent {
                request_id: config.request_id,
                page_id: rng.next_u16() % pool,
                timestamp: ts,
                operation: Operation::Access,
            }
        };
        update_allocated(&mut allocated, &event);
        trace.push(event);
    }
    trace
}

/// ~70% accesses to one hot page, rest random — strong locality test.
fn generate_single_hot_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let mut rng = LcgRng::new(config.seed);
    let hot_page = 1u16;
    let mut trace = Vec::with_capacity(config.num_events);
    let mut ts = 0u16;

    trace.push(AccessEvent {
        request_id: config.request_id,
        page_id: hot_page,
        timestamp: 0,
        operation: Operation::Allocate,
    });

    for i in 1..config.num_events {
        ts = ts.wrapping_add(1);
        let page_id = if i < config.num_events * 7 / 10 {
            hot_page
        } else {
            2 + rng.next_u16() % 10
        };
        trace.push(AccessEvent {
            request_id: config.request_id,
            page_id,
            timestamp: ts,
            operation: Operation::Access,
        });
    }
    trace
}

/// Rotating accesses among pages {1,2,3} — multiple simultaneous hot pages.
fn generate_multiple_hot_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let hot_pages = [1u16, 2, 3];
    let mut trace = Vec::with_capacity(config.num_events);
    let mut ts = 0u16;

    for &p in &hot_pages {
        trace.push(AccessEvent {
            request_id: config.request_id,
            page_id: p,
            timestamp: ts,
            operation: Operation::Allocate,
        });
        ts = ts.wrapping_add(1);
    }

    for i in 0..(config.num_events - hot_pages.len()) {
        ts = ts.wrapping_add(1);
        let page_id = hot_pages[i % hot_pages.len()];
        trace.push(AccessEvent {
            request_id: config.request_id,
            page_id,
            timestamp: ts,
            operation: Operation::Access,
        });
    }
    trace
}

/// Bursts on page bases 100→200→300→100 — tests recency/aging adaptation.
fn generate_bursty_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let bursts = [
        (100u16, 5usize),
        (200, 5),
        (300, 5),
        (100, 5),
    ];
    let mut trace = Vec::with_capacity(config.num_events);
    let mut ts = 0u16;
    let mut idx = 0;

    for &(page_base, burst_len) in &bursts {
        trace.push(AccessEvent {
            request_id: config.request_id,
            page_id: page_base,
            timestamp: ts,
            operation: Operation::Allocate,
        });
        ts = ts.wrapping_add(1);

        for j in 0..burst_len {
            if trace.len() >= config.num_events {
                return trace;
            }
            trace.push(AccessEvent {
                request_id: config.request_id,
                page_id: page_base + (j as u16),
                timestamp: ts,
                operation: Operation::Access,
            });
            ts = ts.wrapping_add(1);
            idx += 1;
        }
        let _ = idx;
    }

    while trace.len() < config.num_events {
        trace.push(AccessEvent {
            request_id: config.request_id,
            page_id: 100,
            timestamp: ts,
            operation: Operation::Access,
        });
        ts = ts.wrapping_add(1);
    }
    trace
}

/// Zipf-skewed page popularity (alpha from config) — approximates real cache skew.
fn generate_zipf_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let n = config.page_pool.max(2) as usize;
    let weights = zipf_weights(n, config.zipf_alpha);
    let mut rng = LcgRng::new(config.seed);
    let mut trace = Vec::with_capacity(config.num_events);
    let mut allocated = std::collections::HashSet::new();
    let mut ts = 0u16;

    for _ in 0..config.num_events {
        ts = ts.wrapping_add(1);
        let page_idx = weighted_pick(&mut rng, &weights);
        let page_id = (page_idx + 1) as u16;

        let event = if !allocated.contains(&page_id) && allocated.len() < NUM_PAGES {
            allocated.insert(page_id);
            AccessEvent {
                request_id: config.request_id,
                page_id,
                timestamp: ts,
                operation: Operation::Allocate,
            }
        } else {
            AccessEvent {
                request_id: config.request_id,
                page_id,
                timestamp: ts,
                operation: Operation::Access,
            }
        };
        trace.push(event);
    }
    trace
}

/// Build normalized Zipf weight vector: w_i ∝ 1/i^alpha.
fn zipf_weights(n: usize, alpha: f64) -> Vec<f64> {
    let mut weights = Vec::with_capacity(n);
    let mut sum = 0.0;
    for i in 1..=n {
        let w = 1.0 / (i as f64).powf(alpha);
        weights.push(w);
        sum += w;
    }
    for w in &mut weights {
        *w /= sum;
    }
    weights
}

/// Sample index from discrete weight distribution.
fn weighted_pick(rng: &mut LcgRng, weights: &[f64]) -> usize {
    let r = rng.next_f64();
    let mut cumulative = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if r <= cumulative {
            return i;
        }
    }
    weights.len().saturating_sub(1)
}

/// Agentic inference pattern: system prefix shared across requests, then per-request suffixes.
fn generate_agentic_trace(config: &TraceConfig) -> Vec<AccessEvent> {
    let mut trace = Vec::with_capacity(config.num_events);
    let mut ts = 0u16;
    let prefix_base = 1000u16;
    let suffix_base = 2000u16;

    // Allocate shared prefix pages (system + shared KV prefix).
    for i in 0..config.prefix_len {
        trace.push(AccessEvent {
            request_id: 0,
            page_id: prefix_base + i as u16,
            timestamp: ts,
            operation: Operation::Allocate,
        });
        ts = ts.wrapping_add(1);
    }

  for req in 1..=config.num_requests {
        // Access shared prefix for each request.
        for i in 0..config.prefix_len {
            if trace.len() >= config.num_events {
                return trace;
            }
            trace.push(AccessEvent {
                request_id: req as u16,
                page_id: prefix_base + i as u16,
                timestamp: ts,
                operation: Operation::Access,
            });
            ts = ts.wrapping_add(1);
        }

        // Per-request suffix pages.
        for j in 0..config.suffix_len {
            if trace.len() >= config.num_events {
                return trace;
            }
            let page_id = suffix_base + (req as u16) * 100 + j as u16;
            if j == 0 {
                trace.push(AccessEvent {
                    request_id: req as u16,
                    page_id,
                    timestamp: ts,
                    operation: Operation::Allocate,
                });
            } else {
                trace.push(AccessEvent {
                    request_id: req as u16,
                    page_id,
                    timestamp: ts,
                    operation: Operation::Access,
                });
            }
            ts = ts.wrapping_add(1);
        }
    }

    while trace.len() < config.num_events {
        trace.push(AccessEvent {
            request_id: 1,
            page_id: prefix_base,
            timestamp: ts,
            operation: Operation::Access,
        });
        ts = ts.wrapping_add(1);
    }
    trace
}

/// CP2-style mixed hot/warm/cold workload with alloc/access/release/miss mix.
fn generate_mixed_trace(config: &TraceConfig) -> Vec<AccessEvent> {
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
            TracePattern::Sequential => unreachable!(),
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
                    AccessEvent {
                        request_id: config.request_id,
                        page_id: allocated[rng.next_usize(allocated.len())],
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
                    pick_random_event(&mut rng, config, &mut allocated, ts, op_roll, cold_base)
                }
            }
            _ => {
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

/// Helper for random/bursty patterns: weighted alloc/access/release/miss.
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

/// Track locally which page_ids have been allocated in the generator
/// (mirrors table state for realistic alloc/release sequences).
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

/// Write trace as human-readable lines: A=allocate, C=access, R=release.
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

/// Workloads used by evaluate_all_policies().
pub fn all_workload_patterns() -> Vec<(TracePattern, &'static str)> {
    vec![
        (TracePattern::Sequential, "sequential"),
        (TracePattern::UniformRandom, "uniform_random"),
        (TracePattern::SingleHot, "single_hot"),
        (TracePattern::MultipleHot, "multiple_hot"),
        (TracePattern::Bursty, "bursty"),
        (TracePattern::Zipf, "zipf"),
        (TracePattern::Agentic, "agentic"),
    ]
}

/// Zipf alpha values from checkpoint 3 spec.
pub const ZIPF_ALPHAS: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

/// Aging interval sweep values from checkpoint 3 spec.
pub const AGING_INTERVALS: [u32; 7] = [16, 32, 64, 128, 256, 512, 1024];
