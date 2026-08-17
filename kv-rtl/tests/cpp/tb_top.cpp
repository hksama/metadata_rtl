// Smoke test for checkpoint 1 single-page metadata tracker.

#include "simulator.h"

#include <cstdio>
#include <cstdlib>

namespace {

int fail(const char* msg) {
    std::fprintf(stderr, "FAIL: %s\n", msg);
    return 1;
}

void drive(SimHandle* sim, uint8_t op, uint16_t req, uint16_t page, uint16_t ts) {
    KvEvent ev{};
    ev.operation = op;
    ev.request_id = req;
    ev.page_id = page;
    ev.event_timestamp = ts;
    sim_drive_event(sim, 1, &ev);
}

}  // namespace

int main() {
    SimHandle* sim = sim_create();
    if (!sim) {
        return fail("sim_create returned null");
    }

    if (sim_resp_valid(sim)) {
        return fail("resp_valid asserted after reset");
    }

    drive(sim, 3, 42, 173, 5);  // ALLOCATE
    sim_tick(sim);
    sim_deassert_event(sim);
    sim_tick(sim);

    if (!sim_resp_valid(sim) || sim_resp_error(sim)) {
        return fail("allocate response incorrect");
    }

    KvMetadata meta{};
    sim_read_metadata(sim, &meta);
    if (!meta.valid || meta.page_id != 173 || meta.refcount != 1) {
        return fail("metadata after allocate incorrect");
    }

    drive(sim, 0, 42, 173, 99);  // READ
    sim_tick(sim);
    sim_deassert_event(sim);
    sim_tick(sim);

    sim_read_metadata(sim, &meta);
    if (meta.access_count != 1 || meta.last_access != 99) {
        return fail("metadata after read incorrect");
    }

    std::printf("PASS: C++ checkpoint 1 smoke tests (%llu cycles)\n",
                static_cast<unsigned long long>(sim_cycle_count(sim)));

    sim_destroy(sim);
    return 0;
}
