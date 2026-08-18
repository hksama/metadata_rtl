// Shared parameters and encodings for KV page metadata RTL.

package kv_pkg;

    parameter int DEFAULT_NUM_PAGES  = 16;
    parameter int PAGE_ID_WIDTH    = 16;
    parameter int REQUEST_ID_WIDTH = 16;
    parameter int COUNTER_WIDTH    = 16;
    parameter int TIMESTAMP_WIDTH  = 16;
    parameter int REFCOUNT_WIDTH   = 8;
    parameter int OP_WIDTH         = 3;
    parameter int STATE_WIDTH      = 2;

    function automatic int entry_idx_width(input int num_pages);
        return (num_pages <= 1) ? 1 : $clog2(num_pages);
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

endpackage
