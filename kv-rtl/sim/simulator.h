// C ABI for the Verilator-backed KV metadata table simulator.
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define KV_NUM_PAGES 16

typedef struct SimHandle SimHandle;

typedef struct {
    uint8_t  operation;
    uint16_t request_id;
    uint16_t page_id;
    uint16_t event_timestamp;
} KvEvent;

typedef struct {
    uint8_t  valid;
    uint16_t page_id;
    uint16_t request_id;
    uint16_t access_count;
    uint16_t last_access;
    uint8_t  refcount;
    uint8_t  state;
} KvMetadata;

SimHandle* sim_create(void);
SimHandle* sim_create_traced(const char* path);
void       sim_destroy(SimHandle* sim);

void sim_reset(SimHandle* sim);
void sim_tick(SimHandle* sim);

void sim_drive_event(SimHandle* sim, int event_valid, const KvEvent* event);
void sim_deassert_event(SimHandle* sim);

int      sim_resp_valid(const SimHandle* sim);
int      sim_resp_error(const SimHandle* sim);
int      sim_resp_hit(const SimHandle* sim);
int      sim_resp_miss(const SimHandle* sim);
uint32_t sim_resp_entry_id(const SimHandle* sim);
void     sim_read_resp_metadata(const SimHandle* sim, KvMetadata* out);

int      sim_num_pages(void);
void     sim_read_table_entry(SimHandle* sim, uint32_t index, KvMetadata* out);

uint32_t sim_hw_timestamp(const SimHandle* sim);
uint64_t sim_cycle_count(const SimHandle* sim);

void sim_finalize_trace(SimHandle* sim);

#ifdef __cplusplus
}
#endif
