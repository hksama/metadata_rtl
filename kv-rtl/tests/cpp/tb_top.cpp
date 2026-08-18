// Smoke test for checkpoint 2 multi-page metadata table.

#include "simulator.h"

#include <cstdio>

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

void submit(SimHandle* sim) {
    sim_tick(sim);
    sim_deassert_event(sim);
    sim_tick(sim);
}

}  // namespace

int main() {
    SimHandle* sim = sim_create();
    if (!sim) {
        return fail("sim_create returned null");
    }

    drive(sim, 3, 1, 10, 0);  // ALLOCATE page 10
    submit(sim);
    if (!sim_resp_valid(sim) || sim_resp_error(sim)) {
        return fail("allocate page 10 failed");
    }

    drive(sim, 3, 1, 11, 1);  // ALLOCATE page 11
    submit(sim);

    drive(sim, 4, 1, 10, 5);  // ACCESS page 10
    submit(sim);
    if (!sim_resp_hit(sim) || sim_resp_miss(sim)) {
        return fail("access hit expected");
    }

    KvMetadata meta{};
    sim_read_resp_metadata(sim, &meta);
    if (meta.access_count != 1) {
        return fail("access count incorrect");
    }
    if (sim_resp_hotness_score(sim) != 1) {
        return fail("hotness score should equal access count for frequency policy");
    }

    drive(sim, 4, 1, 99, 6);  // ACCESS miss
    submit(sim);
    if (!sim_resp_miss(sim) || sim_resp_hit(sim)) {
        return fail("access miss expected");
    }

    std::printf("PASS: C++ checkpoint 3 smoke tests (%llu cycles)\n",
                static_cast<unsigned long long>(sim_cycle_count(sim)));

    sim_destroy(sim);
    return 0;
}
