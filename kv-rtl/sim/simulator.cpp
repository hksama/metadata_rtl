#include "simulator.h"

#include "Vtop.h"
#include "verilated.h"

#if VM_TRACE
#  if VM_TRACE_FST
#    include "verilated_fst_c.h"
#  else
#    include "verilated_vcd_c.h"
#  endif
#endif

#include <cstdint>

struct SimHandle {
    Vtop*            top;
    uint64_t         cycle_count;
#if VM_TRACE
#  if VM_TRACE_FST
    VerilatedFstC* trace;
#  else
    VerilatedVcdC* trace;
#  endif
    bool tracing;
#endif
};

static void clear_event_inputs(SimHandle* sim) {
    sim->top->event_valid = 0;
    sim->top->request_id = 0;
    sim->top->page_id = 0;
    sim->top->event_timestamp = 0;
    sim->top->operation = 0;
}

static void apply_reset(SimHandle* sim, int cycles) {
    sim->top->rst = 1;
    clear_event_inputs(sim);
    sim->top->table_read = 0;
    sim->top->table_idx = 0;
    for (int i = 0; i < cycles; ++i) {
        sim->top->clk = 0;
        sim->top->eval();
#if VM_TRACE
        if (sim->tracing) {
            sim->trace->dump(static_cast<vluint64_t>(sim->cycle_count * 2));
        }
#endif
        sim->top->clk = 1;
        sim->top->eval();
#if VM_TRACE
        if (sim->tracing) {
            sim->trace->dump(static_cast<vluint64_t>(sim->cycle_count * 2 + 1));
        }
#endif
        ++sim->cycle_count;
    }
    sim->top->rst = 0;
}

static SimHandle* sim_create_impl(const char* trace_path) {
#if VM_TRACE
    if (trace_path && trace_path[0] != '\0') {
        Verilated::traceEverOn(true);
    }
#endif

    auto* sim = new SimHandle{};
    sim->top = new Vtop{};
    sim->cycle_count = 0;
#if VM_TRACE
    sim->trace = nullptr;
    sim->tracing = false;
    if (trace_path && trace_path[0] != '\0') {
#  if VM_TRACE_FST
        sim->trace = new VerilatedFstC{};
#  else
        sim->trace = new VerilatedVcdC{};
#  endif
        sim->top->trace(sim->trace, 99);
        sim->trace->open(trace_path);
        sim->tracing = true;
    }
#endif
    Verilated::randReset(0);
    apply_reset(sim, 2);
    return sim;
}

static void read_metadata_fields(
    uint8_t valid,
    uint16_t page_id,
    uint16_t request_id,
    uint16_t access_count,
    uint16_t last_access,
    uint8_t refcount,
    uint8_t state,
    KvMetadata* out
) {
    out->valid = valid;
    out->page_id = page_id;
    out->request_id = request_id;
    out->access_count = access_count;
    out->last_access = last_access;
    out->refcount = refcount;
    out->state = state;
}

extern "C" {

SimHandle* sim_create(void) {
    return sim_create_impl(nullptr);
}

SimHandle* sim_create_traced(const char* path) {
    return sim_create_impl(path);
}

void sim_destroy(SimHandle* sim) {
    if (!sim) {
        return;
    }
#if VM_TRACE
    if (sim->trace) {
        sim->trace->close();
        delete sim->trace;
    }
#endif
    delete sim->top;
    delete sim;
}

void sim_reset(SimHandle* sim) {
    apply_reset(sim, 2);
}

void sim_tick(SimHandle* sim) {
    sim->top->clk = 0;
    sim->top->eval();
#if VM_TRACE
    if (sim->tracing) {
        sim->trace->dump(static_cast<vluint64_t>(sim->cycle_count * 2));
    }
#endif

    sim->top->clk = 1;
    sim->top->eval();
#if VM_TRACE
    if (sim->tracing) {
        sim->trace->dump(static_cast<vluint64_t>(sim->cycle_count * 2 + 1));
    }
#endif
    ++sim->cycle_count;
}

void sim_drive_event(SimHandle* sim, int event_valid, const KvEvent* event) {
    sim->top->event_valid = event_valid ? 1 : 0;
    if (event) {
        sim->top->request_id = event->request_id;
        sim->top->page_id = event->page_id;
        sim->top->event_timestamp = event->event_timestamp;
        sim->top->operation = event->operation;
    }
}

void sim_deassert_event(SimHandle* sim) {
    clear_event_inputs(sim);
}

int sim_resp_valid(const SimHandle* sim) {
    return sim->top->resp_valid ? 1 : 0;
}

int sim_resp_error(const SimHandle* sim) {
    return sim->top->resp_error ? 1 : 0;
}

int sim_resp_hit(const SimHandle* sim) {
    return sim->top->resp_hit ? 1 : 0;
}

int sim_resp_miss(const SimHandle* sim) {
    return sim->top->resp_miss ? 1 : 0;
}

uint32_t sim_resp_entry_id(const SimHandle* sim) {
    return static_cast<uint32_t>(sim->top->resp_entry_id);
}

void sim_read_resp_metadata(const SimHandle* sim, KvMetadata* out) {
    read_metadata_fields(
        sim->top->resp_meta_valid ? 1 : 0,
        static_cast<uint16_t>(sim->top->resp_meta_page_id),
        static_cast<uint16_t>(sim->top->resp_meta_request_id),
        static_cast<uint16_t>(sim->top->resp_meta_access_count),
        static_cast<uint16_t>(sim->top->resp_meta_last_access),
        static_cast<uint8_t>(sim->top->resp_meta_refcount),
        static_cast<uint8_t>(sim->top->resp_meta_state),
        out
    );
}

int sim_num_pages(void) {
    return KV_NUM_PAGES;
}

void sim_read_table_entry(SimHandle* sim, uint32_t index, KvMetadata* out) {
    sim->top->table_read = 1;
    sim->top->table_idx = index;
    sim->top->eval();
    read_metadata_fields(
        sim->top->table_valid ? 1 : 0,
        static_cast<uint16_t>(sim->top->table_page_id),
        static_cast<uint16_t>(sim->top->table_request_id),
        static_cast<uint16_t>(sim->top->table_access_count),
        static_cast<uint16_t>(sim->top->table_last_access),
        static_cast<uint8_t>(sim->top->table_refcount),
        static_cast<uint8_t>(sim->top->table_state),
        out
    );
    sim->top->table_read = 0;
    sim->top->eval();
}

uint32_t sim_hw_timestamp(const SimHandle* sim) {
    return static_cast<uint32_t>(sim->top->hw_timestamp);
}

uint64_t sim_cycle_count(const SimHandle* sim) {
    return sim->cycle_count;
}

void sim_finalize_trace(SimHandle* sim) {
#if VM_TRACE
    if (sim->trace) {
        sim->trace->close();
        delete sim->trace;
        sim->trace = nullptr;
        sim->tracing = false;
    }
#else
    (void)sim;
#endif
}

}  // extern "C"
