# Traces

Synthetic and recorded access traces for KV-cache metadata evaluation will live here.

## Checkpoint 1

Waveforms from the single-page metadata tracker:

- `checkpoint1.vcd` — default from `make simulate`
- `checkpoint1.fst` — when `WAVE_FMT=fst make simulate`

Key signals under `top`: `event_valid`, `operation`, `page_id`, `request_id`,
`event_timestamp`, `resp_valid`, `resp_error`, `meta_valid`, `meta_access_count`,
`meta_refcount`, `hw_timestamp`.

## Checkpoint 0 (historical)

Earlier infrastructure smoke tests used `checkpoint0.vcd`.

## Viewing waveforms

```bash
gtkwave traces/synthetic/checkpoint0.vcd
```

In gtkwave, expand the `top` scope and add the signals above to the waveform viewer.

For FST:

```bash
gtkwave traces/synthetic/checkpoint0.fst
```

Generate a fresh waveform:

```bash
make simulate
# or
WAVE_FMT=fst make simulate
```
