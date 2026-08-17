// Checkpoint 0 infrastructure test — NOT part of the KV-cache accelerator.
// Trivial deterministic transform: output = input + 1, one cycle after input_valid.

module top (
    input  logic        clk,
    input  logic        rst,
    input  logic        input_valid,
    input  logic [31:0] input_data,
    output logic        output_valid,
    output logic [31:0] output_data
);

    logic        input_valid_q;
    logic [31:0] input_data_q;

    always_ff @(posedge clk) begin
        if (rst) begin
            input_valid_q <= 1'b0;
            input_data_q  <= 32'h0;
            output_valid  <= 1'b0;
            output_data   <= 32'h0;
        end else begin
            input_valid_q <= input_valid;
            input_data_q  <= input_data;
            output_valid  <= input_valid_q;
            output_data   <= input_data_q + 32'd1;
        end
    end

endmodule
