#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT}/build"
TRACE_DIR="${ROOT}/traces/synthetic"
WAVE_FMT="${WAVE_FMT:-vcd}"

"${ROOT}/scripts/verilate.sh"

mkdir -p "${TRACE_DIR}"

if [[ "${WAVE_FMT}" == "fst" ]]; then
    WAVE_FILE="${TRACE_DIR}/checkpoint2.fst"
else
    WAVE_FILE="${TRACE_DIR}/checkpoint2.vcd"
fi

echo "Running C++ cycle-by-cycle testbench..."
"${BUILD_DIR}/tb_top"

echo "Running Rust harness (waveform -> ${WAVE_FILE})..."
cd "${ROOT}/rust"
KV_RTL_WAVE="${WAVE_FILE}" cargo run --release

echo ""
echo "Waveform: ${WAVE_FILE}"
echo "Open with gtkwave:"
echo "  gtkwave ${WAVE_FILE}"
