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
#include <cstdlib>
#include <cstring>

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

static void apply_reset(SimHandle* sim, int cycles) {
    sim->top->rst = 1;
    sim->top->input_valid = 0;
    sim->top->input_data = 0;
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

void sim_drive(SimHandle* sim, int input_valid, uint32_t input_data) {
    sim->top->input_valid = input_valid ? 1 : 0;
    sim->top->input_data = input_data;
}

int sim_output_valid(const SimHandle* sim) {
    return sim->top->output_valid ? 1 : 0;
}

uint32_t sim_output_data(const SimHandle* sim) {
    return static_cast<uint32_t>(sim->top->output_data);
}

uint64_t sim_cycle_count(const SimHandle* sim) {
    return sim->cycle_count;
}

int sim_enable_trace(SimHandle* sim, const char* path) {
    (void)sim;
    (void)path;
    return -1;
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
