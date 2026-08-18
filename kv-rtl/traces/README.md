# Traces

Synthetic and recorded access traces for KV-cache metadata evaluation.

## Checkpoint 2

| File | Description |
|------|-------------|
| `checkpoint2.vcd` / `.fst` | Waveform from `make simulate` |
| `checkpoint2_trace.txt` | Human-readable workload (`A` alloc, `C` access, `R` release) |
| `checkpoint2_metrics.csv` | events, cycles, hits, misses, allocations, releases |

Key GTKWave signals under `top`: `event_valid`, `operation`, `page_id`, `resp_hit`,
`resp_miss`, `resp_entry_id`, `resp_meta_access_count`, `hw_timestamp`.

## Viewing waveforms

```bash
make simulate
gtkwave traces/synthetic/checkpoint2.vcd
```

FST: `WAVE_FMT=fst make simulate`
