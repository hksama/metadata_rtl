// Cycle-by-cycle C++ testbench for checkpoint 0 infrastructure validation.

#include "simulator.h"

#include <cstdint>
#include <cstdio>
#include <cstdlib>

namespace {

int fail(const char* msg) {
    std::fprintf(stderr, "FAIL: %s\n", msg);
    return 1;
}

void deassert_inputs(SimHandle* sim) {
    sim_drive(sim, 0, 0);
}

}  // namespace

int main() {
    SimHandle* sim = sim_create();
    if (!sim) {
        return fail("sim_create returned null");
    }

    // After reset, output must be idle.
    if (sim_output_valid(sim)) {
        return fail("output_valid asserted immediately after reset");
    }

    // Drive one transaction; expect result one cycle later.
    const uint32_t input = 41;
    const uint32_t expected = input + 1;

    sim_drive(sim, 1, input);
    sim_tick(sim);

    if (sim_output_valid(sim)) {
        return fail("output_valid asserted too early (latency violation)");
    }

    deassert_inputs(sim);
    sim_tick(sim);

    if (!sim_output_valid(sim)) {
        return fail("output_valid not asserted after expected latency");
    }
    if (sim_output_data(sim) != expected) {
        return fail("incorrect output_data after single transaction");
    }

    // Idle cycle should deassert output_valid.
    sim_tick(sim);
    if (sim_output_valid(sim)) {
        return fail("output_valid stuck high without input_valid");
    }

    // Reset should clear state again.
    sim_drive(sim, 1, 7);
    sim_tick(sim);
    sim_reset(sim);

    if (sim_output_valid(sim)) {
        return fail("output_valid asserted after reset");
    }

    // Back-to-back transactions.
    sim_drive(sim, 1, 10);
    sim_tick(sim);
    deassert_inputs(sim);
    sim_tick(sim);
    if (!sim_output_valid(sim) || sim_output_data(sim) != 11) {
        return fail("first back-to-back transaction incorrect");
    }

    sim_drive(sim, 1, 20);
    sim_tick(sim);
    deassert_inputs(sim);
    sim_tick(sim);
    if (!sim_output_valid(sim) || sim_output_data(sim) != 21) {
        return fail("second back-to-back transaction incorrect");
    }

    std::printf("PASS: C++ cycle-by-cycle tests (%llu cycles)\n",
                static_cast<unsigned long long>(sim_cycle_count(sim)));

    sim_destroy(sim);
    return 0;
}
