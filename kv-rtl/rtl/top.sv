// Top-level wrapper for checkpoint 2 multi-page metadata engine.

import kv_pkg::*;

module top #(
    parameter int NUM_PAGES = kv_pkg::DEFAULT_NUM_PAGES
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
    output logic [kv_pkg::entry_idx_width(NUM_PAGES)-1:0]  resp_entry_id,

    output logic                          resp_meta_valid,
    output logic [PAGE_ID_WIDTH-1:0]     resp_meta_page_id,
    output logic [REQUEST_ID_WIDTH-1:0] resp_meta_request_id,
    output logic [COUNTER_WIDTH-1:0]    resp_meta_access_count,
    output logic [TIMESTAMP_WIDTH-1:0]  resp_meta_last_access,
    output logic [REFCOUNT_WIDTH-1:0]   resp_meta_refcount,
    output logic [STATE_WIDTH-1:0]      resp_meta_state,

    input  logic                          table_read,
    input  logic [kv_pkg::entry_idx_width(NUM_PAGES)-1:0]  table_idx,
    output logic                          table_valid,
    output logic [PAGE_ID_WIDTH-1:0]     table_page_id,
    output logic [REQUEST_ID_WIDTH-1:0] table_request_id,
    output logic [COUNTER_WIDTH-1:0]    table_access_count,
    output logic [TIMESTAMP_WIDTH-1:0]  table_last_access,
    output logic [REFCOUNT_WIDTH-1:0]   table_refcount,
    output logic [STATE_WIDTH-1:0]      table_state,

    output logic [TIMESTAMP_WIDTH-1:0]  hw_timestamp
);

    kv_metadata_engine #(
        .NUM_PAGES(NUM_PAGES)
    ) u_engine (
        .clk                  (clk),
        .rst                  (rst),
        .event_valid          (event_valid),
        .request_id           (request_id),
        .page_id              (page_id),
        .event_timestamp      (event_timestamp),
        .operation            (operation),
        .resp_valid           (resp_valid),
        .resp_error           (resp_error),
        .resp_hit             (resp_hit),
        .resp_miss            (resp_miss),
        .resp_entry_id        (resp_entry_id),
        .resp_meta_valid      (resp_meta_valid),
        .resp_meta_page_id    (resp_meta_page_id),
        .resp_meta_request_id (resp_meta_request_id),
        .resp_meta_access_count(resp_meta_access_count),
        .resp_meta_last_access(resp_meta_last_access),
        .resp_meta_refcount   (resp_meta_refcount),
        .resp_meta_state      (resp_meta_state),
        .table_read           (table_read),
        .table_idx            (table_idx),
        .table_valid          (table_valid),
        .table_page_id        (table_page_id),
        .table_request_id     (table_request_id),
        .table_access_count   (table_access_count),
        .table_last_access    (table_last_access),
        .table_refcount       (table_refcount),
        .table_state          (table_state),
        .hw_timestamp         (hw_timestamp)
    );

endmodule
