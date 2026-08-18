#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT}/build"
VERILATOR_DIR="${BUILD_DIR}/verilator"
RTL_FILES=(
    "${ROOT}/rtl/kv_pkg.sv"
    "${ROOT}/rtl/kv_metadata_engine.sv"
    "${ROOT}/rtl/top.sv"
)
NUM_PAGES="${NUM_PAGES:-16}"
VERILATOR_INCLUDE="${VERILATOR_INCLUDE:-/usr/share/verilator/include}"

mkdir -p "${BUILD_DIR}"

TRACE_FMT="${TRACE_FMT:-vcd}"
TRACE_CFLAGS=""
if [[ "${TRACE_FMT}" == "fst" ]]; then
    TRACE_CFLAGS="-DVM_TRACE_FST"
fi

verilator \
    -Wall -Wno-DECLFILENAME -Wno-IMPORTSTAR \
    --cc \
    -Mdir "${VERILATOR_DIR}" \
    -GNUM_PAGES="${NUM_PAGES}" \
    --top-module top \
    --trace \
    "${RTL_FILES[@]}"

make -C "${VERILATOR_DIR}" -f Vtop.mk VM_PARALLEL_BUILDS=1 -j"$(nproc)"

g++ -std=c++17 -O2 -fPIC \
    -I"${VERILATOR_DIR}" \
    -I"${ROOT}/sim" \
    -I"${VERILATOR_INCLUDE}" \
    -DVM_TRACE=1 ${TRACE_CFLAGS} \
    -c "${ROOT}/sim/simulator.cpp" \
    -o "${BUILD_DIR}/simulator.o"

ar rcs "${BUILD_DIR}/libkvsim.a" "${BUILD_DIR}/simulator.o"

g++ -std=c++17 -O2 \
    -I"${ROOT}/sim" \
    "${ROOT}/tests/cpp/tb_top.cpp" \
    "${BUILD_DIR}/libkvsim.a" \
    "${VERILATOR_DIR}/Vtop__ALL.a" \
    "${VERILATOR_DIR}/libverilated.a" \
    -I"${VERILATOR_DIR}" \
    -I"${VERILATOR_INCLUDE}" \
    -DVM_TRACE=1 ${TRACE_CFLAGS} \
    -o "${BUILD_DIR}/tb_top"

echo "Verilator model and C++ testbench built under ${BUILD_DIR}"
