// Multi-page KV metadata table engine (checkpoint 2).

import kv_pkg::*;

module kv_metadata_engine #(
    parameter int NUM_PAGES = kv_pkg::DEFAULT_NUM_PAGES,
    parameter int ENTRY_IDX_WIDTH = kv_pkg::entry_idx_width(NUM_PAGES)
) (
    input  logic                          clk,
    input  logic                          rst,

    input  logic                          event_valid,
    input  logic [REQUEST_ID_WIDTH-1:0] request_id,
    input  logic [PAGE_ID_WIDTH-1:0]     page_id,
    input  logic [TIMESTAMP_WIDTH-1:0]   event_timestamp,
    input  logic [OP_WIDTH-1:0]          operation,

    output logic                          resp_valid,
    output logic                          resp_error,
    output logic                          resp_hit,
    output logic                          resp_miss,
    output logic [ENTRY_IDX_WIDTH-1:0]  resp_entry_id,

    output logic                          resp_meta_valid,
    output logic [PAGE_ID_WIDTH-1:0]     resp_meta_page_id,
    output logic [REQUEST_ID_WIDTH-1:0] resp_meta_request_id,
    output logic [COUNTER_WIDTH-1:0]    resp_meta_access_count,
    output logic [TIMESTAMP_WIDTH-1:0]  resp_meta_last_access,
    output logic [REFCOUNT_WIDTH-1:0]   resp_meta_refcount,
    output logic [STATE_WIDTH-1:0]      resp_meta_state,

    input  logic                          table_read,
    input  logic [ENTRY_IDX_WIDTH-1:0]  table_idx,
    output logic                          table_valid,
    output logic [PAGE_ID_WIDTH-1:0]     table_page_id,
    output logic [REQUEST_ID_WIDTH-1:0] table_request_id,
    output logic [COUNTER_WIDTH-1:0]    table_access_count,
    output logic [TIMESTAMP_WIDTH-1:0]  table_last_access,
    output logic [REFCOUNT_WIDTH-1:0]   table_refcount,
    output logic [STATE_WIDTH-1:0]      table_state,

    output logic [TIMESTAMP_WIDTH-1:0]  hw_timestamp
);

    logic                          ev_pending;
    logic [REQUEST_ID_WIDTH-1:0] ev_request_id;
    logic [PAGE_ID_WIDTH-1:0]    ev_page_id;
    logic [TIMESTAMP_WIDTH-1:0]  ev_timestamp;
    logic [OP_WIDTH-1:0]         ev_operation;

    logic [NUM_PAGES-1:0]                    entry_valid;
    logic [PAGE_ID_WIDTH-1:0]              entry_page_id     [NUM_PAGES];
    logic [REQUEST_ID_WIDTH-1:0]           entry_request_id  [NUM_PAGES];
    logic [COUNTER_WIDTH-1:0]              entry_access_count[NUM_PAGES];
    logic [TIMESTAMP_WIDTH-1:0]            entry_last_access [NUM_PAGES];
    logic [REFCOUNT_WIDTH-1:0]             entry_refcount    [NUM_PAGES];
    logic [STATE_WIDTH-1:0]                entry_state       [NUM_PAGES];

  logic [NUM_PAGES-1:0]                    next_entry_valid;
  logic [PAGE_ID_WIDTH-1:0]              next_entry_page_id     [NUM_PAGES];
  logic [REQUEST_ID_WIDTH-1:0]           next_entry_request_id  [NUM_PAGES];
  logic [COUNTER_WIDTH-1:0]              next_entry_access_count[NUM_PAGES];
  logic [TIMESTAMP_WIDTH-1:0]            next_entry_last_access [NUM_PAGES];
  logic [REFCOUNT_WIDTH-1:0]             next_entry_refcount    [NUM_PAGES];
  logic [STATE_WIDTH-1:0]                next_entry_state       [NUM_PAGES];

    logic                          resp_meta_valid_r;
    logic [PAGE_ID_WIDTH-1:0]    resp_meta_page_id_r;
    logic [REQUEST_ID_WIDTH-1:0] resp_meta_request_id_r;
    logic [COUNTER_WIDTH-1:0]    resp_meta_access_count_r;
    logic [TIMESTAMP_WIDTH-1:0]  resp_meta_last_access_r;
    logic [REFCOUNT_WIDTH-1:0]   resp_meta_refcount_r;
    logic [STATE_WIDTH-1:0]      resp_meta_state_r;

    logic                          lk_found;
    logic [ENTRY_IDX_WIDTH-1:0]  lk_entry_id;
    logic                          free_found;
    logic [ENTRY_IDX_WIDTH-1:0]  free_entry_id;

    logic                          next_error;
    logic                          next_hit;
    logic                          next_miss;
    logic [ENTRY_IDX_WIDTH-1:0]  next_entry_id;
    logic                          next_meta_valid;
    logic [PAGE_ID_WIDTH-1:0]    next_meta_page_id;
    logic [REQUEST_ID_WIDTH-1:0] next_meta_request_id;
    logic [COUNTER_WIDTH-1:0]    next_meta_access_count;
    logic [TIMESTAMP_WIDTH-1:0]  next_meta_last_access;
    logic [REFCOUNT_WIDTH-1:0]   next_meta_refcount;
    logic [STATE_WIDTH-1:0]      next_meta_state;

    integer lk_i;
    integer free_i;
    integer upd_i;
    integer rst_i;
    integer commit_i;

    always_comb begin
        lk_found = 1'b0;
        lk_entry_id = '0;
        for (lk_i = 0; lk_i < NUM_PAGES; lk_i++) begin
            if (entry_valid[lk_i] && (entry_page_id[lk_i] == ev_page_id)) begin
                lk_found = 1'b1;
                lk_entry_id = ENTRY_IDX_WIDTH'(lk_i);
            end
        end
    end

    always_comb begin
        free_found = 1'b0;
        free_entry_id = '0;
        for (free_i = 0; free_i < NUM_PAGES; free_i++) begin
            if (!entry_valid[free_i] && !free_found) begin
                free_found = 1'b1;
                free_entry_id = ENTRY_IDX_WIDTH'(free_i);
            end
        end
    end

    always_comb begin
        for (upd_i = 0; upd_i < NUM_PAGES; upd_i++) begin
            next_entry_valid[upd_i]        = entry_valid[upd_i];
            next_entry_page_id[upd_i]      = entry_page_id[upd_i];
            next_entry_request_id[upd_i]   = entry_request_id[upd_i];
            next_entry_access_count[upd_i] = entry_access_count[upd_i];
            next_entry_last_access[upd_i]  = entry_last_access[upd_i];
            next_entry_refcount[upd_i]     = entry_refcount[upd_i];
            next_entry_state[upd_i]        = entry_state[upd_i];
        end

        next_error            = 1'b0;
        next_hit              = 1'b0;
        next_miss             = 1'b0;
        next_entry_id         = '0;
        next_meta_valid       = 1'b0;
        next_meta_page_id     = '0;
        next_meta_request_id  = '0;
        next_meta_access_count = '0;
        next_meta_last_access = '0;
        next_meta_refcount    = '0;
        next_meta_state       = STATE_INVALID;

        if (ev_pending) begin
            unique case (ev_operation)
                OP_ALLOCATE: begin
                    if (lk_found) begin
                        next_error = 1'b1;
                    end else if (!free_found) begin
                        next_error = 1'b1;
                    end else begin
                        next_entry_valid[free_entry_id]        = 1'b1;
                        next_entry_page_id[free_entry_id]      = ev_page_id;
                        next_entry_request_id[free_entry_id]   = ev_request_id;
                        next_entry_access_count[free_entry_id] = '0;
                        next_entry_last_access[free_entry_id]  = '0;
                        next_entry_refcount[free_entry_id]     = sat_inc_ref('0);
                        next_entry_state[free_entry_id]        = STATE_VALID;
                        next_entry_id                          = free_entry_id;
                        next_meta_valid                        = 1'b1;
                        next_meta_page_id                      = ev_page_id;
                        next_meta_request_id                   = ev_request_id;
                        next_meta_access_count                 = '0;
                        next_meta_last_access                  = '0;
                        next_meta_refcount                     = sat_inc_ref('0);
                        next_meta_state                        = STATE_VALID;
                    end
                end

                OP_READ, OP_WRITE, OP_ACCESS: begin
                    if (!lk_found) begin
                        next_miss = 1'b1;
                    end else begin
                        next_hit = 1'b1;
                        next_entry_id = lk_entry_id;
                        next_entry_access_count[lk_entry_id] =
                            sat_inc_u(entry_access_count[lk_entry_id]);
                        next_entry_last_access[lk_entry_id] = ev_timestamp;
                        next_meta_valid        = 1'b1;
                        next_meta_page_id      = entry_page_id[lk_entry_id];
                        next_meta_request_id   = entry_request_id[lk_entry_id];
                        next_meta_access_count = sat_inc_u(entry_access_count[lk_entry_id]);
                        next_meta_last_access  = ev_timestamp;
                        next_meta_refcount     = entry_refcount[lk_entry_id];
                        next_meta_state        = entry_state[lk_entry_id];
                    end
                end

                OP_RELEASE: begin
                    if (!lk_found) begin
                        next_error = 1'b1;
                    end else if (entry_refcount[lk_entry_id] == '0) begin
                        next_error = 1'b1;
                    end else begin
                        next_hit = 1'b1;
                        next_entry_id = lk_entry_id;
                        next_entry_refcount[lk_entry_id] =
                            sat_dec_ref(entry_refcount[lk_entry_id]);
                        next_meta_valid        = 1'b1;
                        next_meta_page_id      = entry_page_id[lk_entry_id];
                        next_meta_request_id   = entry_request_id[lk_entry_id];
                        next_meta_access_count = entry_access_count[lk_entry_id];
                        next_meta_last_access  = entry_last_access[lk_entry_id];
                        next_meta_refcount     = sat_dec_ref(entry_refcount[lk_entry_id]);
                        next_meta_state        = entry_state[lk_entry_id];
                    end
                end

                default: begin
                    next_error = 1'b1;
                end
            endcase
        end
    end

    always_comb begin
        table_valid        = 1'b0;
        table_page_id      = '0;
        table_request_id   = '0;
        table_access_count = '0;
        table_last_access  = '0;
        table_refcount     = '0;
        table_state        = STATE_INVALID;

        if (table_read) begin
            table_valid        = entry_valid[table_idx];
            table_page_id      = entry_page_id[table_idx];
            table_request_id   = entry_request_id[table_idx];
            table_access_count = entry_access_count[table_idx];
            table_last_access  = entry_last_access[table_idx];
            table_refcount     = entry_refcount[table_idx];
            table_state        = entry_state[table_idx];
        end
    end

    always_ff @(posedge clk) begin
        if (rst) begin
            ev_pending    <= 1'b0;
            ev_request_id <= '0;
            ev_page_id    <= '0;
            ev_timestamp  <= '0;
            ev_operation  <= '0;

            resp_valid    <= 1'b0;
            resp_error    <= 1'b0;
            resp_hit      <= 1'b0;
            resp_miss     <= 1'b0;
            resp_entry_id <= '0;

            resp_meta_valid_r        <= 1'b0;
            resp_meta_page_id_r      <= '0;
            resp_meta_request_id_r   <= '0;
            resp_meta_access_count_r <= '0;
            resp_meta_last_access_r  <= '0;
            resp_meta_refcount_r     <= '0;
            resp_meta_state_r        <= STATE_INVALID;

            for (rst_i = 0; rst_i < NUM_PAGES; rst_i++) begin
                entry_valid[rst_i]        <= 1'b0;
                entry_page_id[rst_i]      <= '0;
                entry_request_id[rst_i]   <= '0;
                entry_access_count[rst_i] <= '0;
                entry_last_access[rst_i]  <= '0;
                entry_refcount[rst_i]     <= '0;
                entry_state[rst_i]        <= STATE_INVALID;
            end

            hw_timestamp <= '0;
        end else begin
            hw_timestamp <= hw_timestamp + {{(TIMESTAMP_WIDTH - 1){1'b0}}, 1'b1};

            resp_valid <= 1'b0;
            resp_error <= 1'b0;
            resp_hit   <= 1'b0;
            resp_miss  <= 1'b0;

            if (ev_pending) begin
                resp_valid    <= 1'b1;
                resp_error    <= next_error;
                resp_hit      <= next_hit;
                resp_miss     <= next_miss;
                resp_entry_id <= next_entry_id;

                resp_meta_valid_r        <= next_meta_valid;
                resp_meta_page_id_r      <= next_meta_page_id;
                resp_meta_request_id_r   <= next_meta_request_id;
                resp_meta_access_count_r <= next_meta_access_count;
                resp_meta_last_access_r  <= next_meta_last_access;
                resp_meta_refcount_r     <= next_meta_refcount;
                resp_meta_state_r        <= next_meta_state;

                for (commit_i = 0; commit_i < NUM_PAGES; commit_i++) begin
                    entry_valid[commit_i]        <= next_entry_valid[commit_i];
                    entry_page_id[commit_i]      <= next_entry_page_id[commit_i];
                    entry_request_id[commit_i]   <= next_entry_request_id[commit_i];
                    entry_access_count[commit_i] <= next_entry_access_count[commit_i];
                    entry_last_access[commit_i]  <= next_entry_last_access[commit_i];
                    entry_refcount[commit_i]     <= next_entry_refcount[commit_i];
                    entry_state[commit_i]        <= next_entry_state[commit_i];
                end

                ev_pending <= 1'b0;
            end else if (event_valid) begin
                ev_request_id <= request_id;
                ev_page_id    <= page_id;
                ev_timestamp  <= event_timestamp;
                ev_operation  <= operation;
                ev_pending    <= 1'b1;
            end
        end
    end

    assign resp_meta_valid        = resp_meta_valid_r;
    assign resp_meta_page_id      = resp_meta_page_id_r;
    assign resp_meta_request_id   = resp_meta_request_id_r;
    assign resp_meta_access_count = resp_meta_access_count_r;
    assign resp_meta_last_access  = resp_meta_last_access_r;
    assign resp_meta_refcount     = resp_meta_refcount_r;
    assign resp_meta_state        = resp_meta_state_r;

endmodule
