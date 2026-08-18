// Hotness scoring engine — logically separated from metadata table storage.
//
// Interface:
//   policy_update(access, metadata) -> updated aging counter (on access)
//   policy_score(metadata, timestamp) -> hotness score
//
// Integer scores only — no floating point (unsuitable for synthesizable RTL).

import kv_pkg::*;

module kv_hotness_engine #(
    parameter int POLICY      = kv_pkg::POLICY_FREQUENCY,
    parameter int SCORE_WIDTH = kv_pkg::DEFAULT_SCORE_WIDTH
) (
    input  logic                          entry_valid,
    input  logic [COUNTER_WIDTH-1:0]      access_count,
    input  logic [TIMESTAMP_WIDTH-1:0]  last_access,
    input  logic [AGING_COUNTER_MAX_WIDTH-1:0] aging_counter_in,

    input  logic                          is_access,
    input  logic                          do_age,

    input  logic [TIMESTAMP_WIDTH-1:0]  current_timestamp,

    output logic [AGING_COUNTER_MAX_WIDTH-1:0] aging_counter_out,
    output logic [SCORE_WIDTH-1:0]           hotness_score
);

    logic [AGING_COUNTER_MAX_WIDTH-1:0] next_aging;
    logic [SCORE_WIDTH-1:0]             score;
    logic [DEFAULT_SCORE_WIDTH-1:0]     score_full;

    always_comb begin
        next_aging = policy_update_aging(
            POLICY,
            aging_counter_in,
            entry_valid && is_access,
            entry_valid && do_age
        );

        score_full = policy_score(
            POLICY,
            entry_valid,
            access_count,
            last_access,
            next_aging,
            current_timestamp
        );
        score = score_full[SCORE_WIDTH-1:0];
    end

    assign aging_counter_out = next_aging;
    assign hotness_score     = score;

endmodule
