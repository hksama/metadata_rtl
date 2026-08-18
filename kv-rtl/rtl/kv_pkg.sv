// Shared parameters and encodings for KV page metadata RTL.

package kv_pkg;

    parameter int DEFAULT_NUM_PAGES     = 16;
    parameter int PAGE_ID_WIDTH        = 16;
    parameter int REQUEST_ID_WIDTH     = 16;
    parameter int COUNTER_WIDTH        = 16;
    parameter int TIMESTAMP_WIDTH      = 16;
    parameter int REFCOUNT_WIDTH       = 8;
    parameter int OP_WIDTH             = 3;
    parameter int STATE_WIDTH          = 2;

    // Checkpoint 3 — hotness policy configuration (overridable at Verilate time).
    parameter int DEFAULT_SCORE_WIDTH       = 8;
    parameter int AGING_COUNTER_MAX_WIDTH   = 4;
    parameter int DEFAULT_AGING_INTERVAL    = 64;

    typedef int policy_e;
    localparam policy_e POLICY_FREQUENCY   = 0;
    localparam policy_e POLICY_AGING_2BIT  = 1;
    localparam policy_e POLICY_AGING_4BIT  = 2;
    localparam policy_e POLICY_APPROX_LRU  = 3;

    function automatic int entry_idx_width(input int num_pages);
        return (num_pages <= 1) ? 1 : $clog2(num_pages);
    endfunction

    function automatic int aging_counter_width(input policy_e policy);
        if (policy == POLICY_AGING_2BIT) begin
            return 2;
        end else if (policy == POLICY_AGING_4BIT) begin
            return 4;
        end else begin
            return 0;
        end
    endfunction

    typedef logic [OP_WIDTH-1:0] op_t;
    localparam op_t OP_READ     = 3'd0;
    localparam op_t OP_WRITE    = 3'd1;
    localparam op_t OP_RELEASE  = 3'd2;
    localparam op_t OP_ALLOCATE = 3'd3;
    localparam op_t OP_ACCESS   = 3'd4;

    typedef logic [STATE_WIDTH-1:0] state_t;
    localparam state_t STATE_INVALID = 2'd0;
    localparam state_t STATE_VALID   = 2'd1;

    function automatic logic is_access_op(input op_t op);
        return (op == OP_READ) || (op == OP_WRITE) || (op == OP_ACCESS);
    endfunction

    function automatic logic [COUNTER_WIDTH-1:0] sat_inc_u(
        input logic [COUNTER_WIDTH-1:0] value
    );
        if (&value) begin
            sat_inc_u = value;
        end else begin
            sat_inc_u = value + {{(COUNTER_WIDTH - 1){1'b0}}, 1'b1};
        end
    endfunction

    function automatic logic [REFCOUNT_WIDTH-1:0] sat_inc_ref(
        input logic [REFCOUNT_WIDTH-1:0] value
    );
        if (&value) begin
            sat_inc_ref = value;
        end else begin
            sat_inc_ref = value + {{(REFCOUNT_WIDTH - 1){1'b0}}, 1'b1};
        end
    endfunction

    function automatic logic [REFCOUNT_WIDTH-1:0] sat_dec_ref(
        input logic [REFCOUNT_WIDTH-1:0] value
    );
        if (value == '0) begin
            sat_dec_ref = '0;
        end else begin
            sat_dec_ref = value - {{(REFCOUNT_WIDTH - 1){1'b0}}, 1'b1};
        end
    endfunction

    function automatic logic [AGING_COUNTER_MAX_WIDTH-1:0] sat_inc_aging(
        input logic [AGING_COUNTER_MAX_WIDTH-1:0] value,
        input logic [AGING_COUNTER_MAX_WIDTH-1:0] max_val
    );
        if (value >= max_val) begin
            sat_inc_aging = max_val;
        end else begin
            sat_inc_aging = value + {{(AGING_COUNTER_MAX_WIDTH - 1){1'b0}}, 1'b1};
        end
    endfunction

    // policy_update: apply access or global aging to the aging counter.
    function automatic logic [AGING_COUNTER_MAX_WIDTH-1:0] policy_update_aging(
        input int policy,
        input logic [AGING_COUNTER_MAX_WIDTH-1:0] aging_in,
        input logic is_access,
        input logic do_age
    );
        logic [AGING_COUNTER_MAX_WIDTH-1:0] next;
        next = aging_in;
        if (do_age) begin
            next = aging_in >> 1;
        end else if (is_access) begin
            unique case (policy)
                POLICY_AGING_2BIT: next = sat_inc_aging(aging_in, 4'd3);
                POLICY_AGING_4BIT: next = sat_inc_aging(aging_in, 4'd15);
                default: next = aging_in;
            endcase
        end
        policy_update_aging = next;
    endfunction

    // policy_score: compute integer hotness from metadata (no floating point).
    // Returns DEFAULT_SCORE_WIDTH bits; callers truncate via parameter if needed.
    function automatic logic [DEFAULT_SCORE_WIDTH-1:0] policy_score(
        input int policy,
        input logic entry_valid,
        input logic [COUNTER_WIDTH-1:0] access_count,
        input logic [TIMESTAMP_WIDTH-1:0] last_access,
        input logic [AGING_COUNTER_MAX_WIDTH-1:0] aging_counter,
        input logic [TIMESTAMP_WIDTH-1:0] current_timestamp
    );
        logic [TIMESTAMP_WIDTH-1:0] age;
        logic [TIMESTAMP_WIDTH-1:0] recency;
        logic [DEFAULT_SCORE_WIDTH-1:0] raw;
        begin
            if (!entry_valid) begin
                policy_score = '0;
            end else begin
                unique case (policy)
                    POLICY_FREQUENCY: begin
                        raw = access_count[DEFAULT_SCORE_WIDTH-1:0];
                    end
                    POLICY_AGING_2BIT: begin
                        raw = {{(DEFAULT_SCORE_WIDTH - 2){1'b0}}, aging_counter[1:0]};
                    end
                    POLICY_AGING_4BIT: begin
                        raw = {{(DEFAULT_SCORE_WIDTH - 4){1'b0}}, aging_counter[3:0]};
                    end
                    POLICY_APPROX_LRU: begin
                        age = current_timestamp - last_access;
                        recency = {{(TIMESTAMP_WIDTH){1'b1}}} - age;
                        raw = recency[DEFAULT_SCORE_WIDTH-1:0];
                    end
                    default: raw = '0;
                endcase
                policy_score = raw;
            end
        end
    endfunction

endpackage
