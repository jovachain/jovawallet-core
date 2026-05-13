#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# macOS: Apple's clang lacks the wasm32-unknown-unknown backend. Use Homebrew LLVM
# if it's available; otherwise let cargo find whatever clang is on PATH. On Linux
# (CI), system clang supports wasm32 natively so no override is needed.
if [[ "$(uname -s)" == "Darwin" ]]; then
  if command -v brew >/dev/null 2>&1; then
    BREW_LLVM_PREFIX="$(brew --prefix llvm 2>/dev/null || true)"
    if [[ -n "$BREW_LLVM_PREFIX" && -x "$BREW_LLVM_PREFIX/bin/clang" ]]; then
      export CC_wasm32_unknown_unknown="$BREW_LLVM_PREFIX/bin/clang"
    fi
  fi
fi

# Build the WASM crate.
# --target nodejs: Node's undici does not support fetch() for file:// URLs, which
# --target web would require. Use nodejs so the init() call uses fs.readFileSync
# instead. (Spike finding: phase -1 wasm-smoke.mjs used the same workaround.)
(cd ../../crates/jova-core-wasm && \
  wasm-pack build --release --target nodejs --out-dir ../../bindings/wasm/pkg)

echo "[ok] WASM package at $(pwd)/pkg"
