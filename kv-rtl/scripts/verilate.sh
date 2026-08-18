#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT}/build"
VERILATOR_DIR="${BUILD_DIR}/verilator"
RTL_FILES=(
    "${ROOT}/rtl/kv_pkg.sv"
    "${ROOT}/rtl/kv_hotness_engine.sv"
    "${ROOT}/rtl/kv_metadata_engine.sv"
    "${ROOT}/rtl/top.sv"
)
NUM_PAGES="${NUM_PAGES:-16}"
POLICY="${POLICY:-FREQUENCY}"
SCORE_WIDTH="${SCORE_WIDTH:-8}"
AGING_INTERVAL="${AGING_INTERVAL:-64}"
VERILATOR_INCLUDE="${VERILATOR_INCLUDE:-/usr/share/verilator/include}"

case "${POLICY}" in
    FREQUENCY|FREQ) POLICY_ID=0 ;;
    AGING2|AGING_2BIT) POLICY_ID=1 ;;
    AGING4|AGING_4BIT) POLICY_ID=2 ;;
    LRU|APPROX_LRU) POLICY_ID=3 ;;
    *) echo "Unknown POLICY=${POLICY}" >&2; exit 1 ;;
esac

mkdir -p "${BUILD_DIR}"

TRACE_FMT="${TRACE_FMT:-vcd}"
TRACE_CFLAGS=""
if [[ "${TRACE_FMT}" == "fst" ]]; then
    TRACE_CFLAGS="-DVM_TRACE_FST"
fi

verilator \
    -Wall -Wno-DECLFILENAME -Wno-IMPORTSTAR -Wno-UNUSEDSIGNAL \
    --cc \
    -Mdir "${VERILATOR_DIR}" \
    -GNUM_PAGES="${NUM_PAGES}" \
    -GPOLICY="${POLICY_ID}" \
    -GSCORE_WIDTH="${SCORE_WIDTH}" \
    -GAGING_INTERVAL="${AGING_INTERVAL}" \
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

echo "Verilator model built: POLICY=${POLICY} SCORE_WIDTH=${SCORE_WIDTH} AGING_INTERVAL=${AGING_INTERVAL}"
