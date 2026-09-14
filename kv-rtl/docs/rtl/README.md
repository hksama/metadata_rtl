# SystemVerilog Reference

Detailed documentation for each RTL source file under `rtl/`.

| File | Document |
|------|----------|
| `kv_pkg.sv` | [kv_pkg.md](kv_pkg.md) — widths, opcodes, policy functions |
| `kv_hotness_engine.sv` | [kv_hotness_engine.md](kv_hotness_engine.md) — per-entry hotness scorer |
| `kv_metadata_engine.sv` | [kv_metadata_engine.md](kv_metadata_engine.md) — metadata table + pipeline |
| `top.sv` | [top.md](top.md) — Verilator top wrapper |

See also:

- [../rtl_architecture.md](../rtl_architecture.md) — high-level RTL overview
- [../protocol.md](../protocol.md) — event/response timing
- [../hotness_policies.md](../hotness_policies.md) — policy selection and experiments
