# Traces

Synthetic and recorded access traces for KV-cache metadata evaluation will live here.

## Checkpoint 0

Waveforms from the infrastructure smoke test are written to `synthetic/`:

- `checkpoint0.vcd` — default VCD output from `make simulate`
- `checkpoint0.fst` — FST output when `WAVE_FMT=fst make simulate`

Signals to inspect: `clk`, `rst`, `input_valid`, `input_data`, `output_valid`, `output_data`.

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
