#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

rm -rf "${ROOT}/build"
rm -rf "${ROOT}/rust/target"

find "${ROOT}/traces/synthetic" -type f \( -name '*.vcd' -o -name '*.fst' \) -delete 2>/dev/null || true

echo "Clean complete."
