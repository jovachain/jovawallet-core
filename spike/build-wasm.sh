#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Baseline (no chains) — browser target
(cd crates/jova-spike-wasm && wasm-pack build --release --target web --out-dir ../../generated/wasm-baseline)

# Baseline — Node.js target (used by wasm-smoke.mjs; fetch() doesn't work in Node for file:// URLs)
(cd crates/jova-spike-wasm && wasm-pack build --release --target nodejs --out-dir ../../generated/wasm-baseline-node)

# Per-chain feature builds — record outcomes; do not exit on failure.
for feat in chain-btc chain-sol chain-xrp; do
  if (cd crates/jova-spike-wasm && wasm-pack build --release --target web \
      --out-dir "../../generated/wasm-$feat" -- --features "$feat"); then
    echo "[ok] $feat"
  else
    echo "[fail] $feat"
  fi
done

# All-chains
if (cd crates/jova-spike-wasm && wasm-pack build --release --target web \
    --out-dir ../../generated/wasm-all -- --features all-chains); then
  echo "[ok] all-chains"
else
  echo "[fail] all-chains"
fi
