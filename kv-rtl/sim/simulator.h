// C ABI for the Verilator-backed simulator. Consumed by Rust via FFI.
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SimHandle SimHandle;

SimHandle* sim_create(void);
SimHandle* sim_create_traced(const char* path);
void       sim_destroy(SimHandle* sim);

void sim_reset(SimHandle* sim);
void sim_tick(SimHandle* sim);

void sim_drive(SimHandle* sim, int input_valid, uint32_t input_data);

int      sim_output_valid(const SimHandle* sim);
uint32_t sim_output_data(const SimHandle* sim);

uint64_t sim_cycle_count(const SimHandle* sim);

// Optional waveform tracing (must be requested at creation time).
int  sim_enable_trace(SimHandle* sim, const char* path);
void sim_finalize_trace(SimHandle* sim);

#ifdef __cplusplus
}
#endif
