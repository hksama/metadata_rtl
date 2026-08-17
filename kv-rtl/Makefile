ROOT        := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
BUILD_DIR   := $(ROOT)/build
VERILATOR_DIR := $(BUILD_DIR)/verilator
RUST_DIR    := $(ROOT)/rust
VERILATOR_INCLUDE ?= /usr/share/verilator/include

export KV_RTL_ROOT := $(ROOT)

.PHONY: all verilate test simulate clean cpp-test rust-test

all: test

verilate:
	./scripts/verilate.sh

cpp-test: verilate
	$(BUILD_DIR)/tb_top

rust-test: verilate
	cd $(RUST_DIR) && cargo test --release
	cd $(RUST_DIR) && cargo run --release

test: verilate cpp-test rust-test
	@echo "All checkpoint 0 tests passed."

simulate:
	./scripts/simulate.sh

clean:
	./scripts/clean.sh
