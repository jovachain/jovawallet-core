#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Build the WASM crate.
# --target nodejs: Node's undici does not support fetch() for file:// URLs, which
# --target web would require. Use nodejs so the init() call uses fs.readFileSync
# instead. (Spike finding: phase -1 wasm-smoke.mjs used the same workaround.)
(cd ../../crates/jova-core-wasm && \
  wasm-pack build --release --target nodejs --out-dir ../../bindings/wasm/pkg)

echo "✅ WASM package at $(pwd)/pkg"
