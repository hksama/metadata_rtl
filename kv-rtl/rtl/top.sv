// Top-level wrapper for checkpoint 1 single-page metadata tracker.

import kv_pkg::*;

module top (
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

    kv_page_tracker u_tracker (
        .clk             (clk),
        .rst             (rst),
        .event_valid     (event_valid),
        .request_id      (request_id),
        .page_id         (page_id),
        .event_timestamp (event_timestamp),
        .operation       (operation),
        .resp_valid      (resp_valid),
        .resp_error      (resp_error),
        .meta_valid      (meta_valid),
        .meta_page_id    (meta_page_id),
        .meta_request_id (meta_request_id),
        .meta_access_count(meta_access_count),
        .meta_last_access(meta_last_access),
        .meta_refcount   (meta_refcount),
        .meta_state      (meta_state),
        .hw_timestamp    (hw_timestamp)
    );

endmodule
