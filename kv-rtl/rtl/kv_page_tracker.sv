// Single-page KV metadata tracker (checkpoint 1).

import kv_pkg::*;

module kv_page_tracker (
    input  logic                          clk,
    input  logic                          rst,

    input  logic                          event_valid,
    input  logic [REQUEST_ID_WIDTH-1:0] request_id,
    input  logic [PAGE_ID_WIDTH-1:0]     page_id,
    input  logic [TIMESTAMP_WIDTH-1:0]   event_timestamp,
    input  logic [OP_WIDTH-1:0]          operation,

    output logic                          resp_valid,
    output logic                          resp_error,

    output logic                          meta_valid,
    output logic [PAGE_ID_WIDTH-1:0]     meta_page_id,
    output logic [REQUEST_ID_WIDTH-1:0] meta_request_id,
    output logic [COUNTER_WIDTH-1:0]    meta_access_count,
    output logic [TIMESTAMP_WIDTH-1:0]  meta_last_access,
    output logic [REFCOUNT_WIDTH-1:0]   meta_refcount,
    output logic [STATE_WIDTH-1:0]      meta_state,

    output logic [TIMESTAMP_WIDTH-1:0]  hw_timestamp
);

    logic                          ev_pending;
    logic [REQUEST_ID_WIDTH-1:0] ev_request_id;
    logic [PAGE_ID_WIDTH-1:0]    ev_page_id;
    logic [TIMESTAMP_WIDTH-1:0]  ev_timestamp;
    logic [OP_WIDTH-1:0]         ev_operation;

    logic                          page_valid;
    logic [PAGE_ID_WIDTH-1:0]    page_page_id;
    logic [REQUEST_ID_WIDTH-1:0] page_request_id;
    logic [COUNTER_WIDTH-1:0]    page_access_count;
    logic [TIMESTAMP_WIDTH-1:0]  page_last_access;
    logic [REFCOUNT_WIDTH-1:0]   page_refcount;
    logic [STATE_WIDTH-1:0]      page_state;

    logic                          next_error;
    logic                          next_page_valid;
    logic [PAGE_ID_WIDTH-1:0]    next_page_page_id;
    logic [REQUEST_ID_WIDTH-1:0] next_page_request_id;
    logic [COUNTER_WIDTH-1:0]    next_page_access_count;
    logic [TIMESTAMP_WIDTH-1:0]  next_page_last_access;
    logic [REFCOUNT_WIDTH-1:0]   next_page_refcount;
    logic [STATE_WIDTH-1:0]      next_page_state;

    function automatic logic page_id_match;
        input logic [PAGE_ID_WIDTH-1:0] stored_id;
        input logic [PAGE_ID_WIDTH-1:0] event_id;
        page_id_match = (stored_id == event_id);
    endfunction

    always_comb begin
        next_error            = 1'b0;
        next_page_valid       = page_valid;
        next_page_page_id     = page_page_id;
        next_page_request_id  = page_request_id;
        next_page_access_count = page_access_count;
        next_page_last_access = page_last_access;
        next_page_refcount    = page_refcount;
        next_page_state       = page_state;

        if (ev_pending) begin
            unique case (ev_operation)
                OP_ALLOCATE: begin
                    if (page_valid) begin
                        next_error = 1'b1;
                    end else begin
                        next_page_valid       = 1'b1;
                        next_page_page_id     = ev_page_id;
                        next_page_request_id  = ev_request_id;
                        next_page_access_count = '0;
                        next_page_last_access = '0;
                        next_page_refcount    = sat_inc_ref('0);
                        next_page_state       = STATE_VALID;
                    end
                end

                OP_READ, OP_WRITE: begin
                    if (!page_valid || !page_id_match(page_page_id, ev_page_id)) begin
                        next_error = 1'b1;
                    end else begin
                        next_page_access_count = sat_inc_u(page_access_count);
                        next_page_last_access  = ev_timestamp;
                    end
                end

                OP_RELEASE: begin
                    if (!page_valid || !page_id_match(page_page_id, ev_page_id)) begin
                        next_error = 1'b1;
                    end else if (page_refcount == '0) begin
                        next_error = 1'b1;
                    end else begin
                        next_page_refcount = sat_dec_ref(page_refcount);
                    end
                end

                default: begin
                    next_error = 1'b1;
                end
            endcase
        end
    end

    always_ff @(posedge clk) begin
        if (rst) begin
            ev_pending        <= 1'b0;
            ev_request_id     <= '0;
            ev_page_id        <= '0;
            ev_timestamp      <= '0;
            ev_operation      <= '0;

            resp_valid        <= 1'b0;
            resp_error        <= 1'b0;

            page_valid        <= 1'b0;
            page_page_id      <= '0;
            page_request_id   <= '0;
            page_access_count <= '0;
            page_last_access  <= '0;
            page_refcount     <= '0;
            page_state        <= STATE_INVALID;

            hw_timestamp      <= '0;
        end else begin
            hw_timestamp <= hw_timestamp + {{(TIMESTAMP_WIDTH - 1){1'b0}}, 1'b1};

            resp_valid <= 1'b0;
            resp_error <= 1'b0;

            if (ev_pending) begin
                resp_valid        <= 1'b1;
                resp_error        <= next_error;
                page_valid        <= next_page_valid;
                page_page_id      <= next_page_page_id;
                page_request_id   <= next_page_request_id;
                page_access_count <= next_page_access_count;
                page_last_access  <= next_page_last_access;
                page_refcount     <= next_page_refcount;
                page_state        <= next_page_state;
                ev_pending        <= 1'b0;
            end else if (event_valid) begin
                ev_request_id <= request_id;
                ev_page_id    <= page_id;
                ev_timestamp  <= event_timestamp;
                ev_operation  <= operation;
                ev_pending    <= 1'b1;
            end
        end
    end

    assign meta_valid        = page_valid;
    assign meta_page_id      = page_page_id;
    assign meta_request_id   = page_request_id;
    assign meta_access_count = page_access_count;
    assign meta_last_access  = page_last_access;
    assign meta_refcount     = page_refcount;
    assign meta_state        = page_state;

endmodule
