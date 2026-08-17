# Checkpoint 0 — Protocol

## Interface

| Signal        | Direction | Width | Description                          |
|---------------|-----------|-------|--------------------------------------|
| `clk`         | input     | 1     | Rising-edge clock                    |
| `rst`         | input     | 1     | Synchronous active-high reset        |
| `input_valid` | input     | 1     | Request qualifier for `input_data`   |
| `input_data`  | input     | 32    | Payload word                         |
| `output_valid`| output    | 1     | Result qualifier for `output_data`   |
| `output_data` | output    | 32    | Transformed payload                  |

## Timing

1. While `rst` is asserted, `output_valid` must be low.
2. When `input_valid` is high in cycle *N*, the result appears in cycle *N+1*
   with `output_valid` high.
3. `output_valid` is only high for one cycle per accepted input.
4. Deasserting `input_valid` on idle cycles must not produce spurious outputs.

## Violations detected by tests

- `output_valid` high during/just after reset
- Result appearing in the same cycle as `input_valid` (latency bug)
- `output_data != input_data + 1`
- `output_valid` stuck high without a preceding qualified input
