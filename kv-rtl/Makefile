ROOT        := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
BUILD_DIR   := $(ROOT)/build
VERILATOR_DIR := $(BUILD_DIR)/verilator
RUST_DIR    := $(ROOT)/rust
VERILATOR_INCLUDE ?= /usr/share/verilator/include

# Checkpoint 3 policy knobs (compile-time selection).
POLICY          ?= FREQUENCY
SCORE_WIDTH     ?= 8
AGING_INTERVAL  ?= 64
NUM_PAGES       ?= 16

export KV_RTL_ROOT := $(ROOT)
export POLICY SCORE_WIDTH AGING_INTERVAL NUM_PAGES

.PHONY: all verilate test simulate clean cpp-test rust-test policy-test aging-sweep

all: test

verilate:
	./scripts/verilate.sh

cpp-test: verilate
	$(BUILD_DIR)/tb_top

rust-test: verilate
	cd $(RUST_DIR) && cargo test --release
	cd $(RUST_DIR) && cargo run --release --bin kv-rtl-harness

test: verilate cpp-test rust-test
	@echo "All checkpoint 3 tests passed."

# Run policy evaluation across all policies.
policy-test: verilate
	cd $(RUST_DIR) && cargo run --release --bin kv-eval -- --all-policies

# Sweep aging intervals for 2-bit and 4-bit policies.
aging-sweep: verilate
	cd $(RUST_DIR) && cargo run --release --bin kv-eval -- --aging-sweep

simulate:
	./scripts/simulate.sh

clean:
	./scripts/clean.sh
