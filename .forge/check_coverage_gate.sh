#!/bin/sh
set -e
echo "=== Running coverage gate ==="
bash scripts/coverage-gate.sh
echo "=== Coverage gate PASSED ==="
