#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

: "${ANDROID_NDK_HOME:=$HOME/Library/Android/sdk/ndk/29.0.14206865}"
export ANDROID_NDK_HOME

cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -o generated/android/jniLibs \
  build -p jova-spike --release --features ffi

# uniffi-bindgen cannot load a cross-compiled Android .so on macOS (ELF != Mach-O).
# Generate Kotlin from the native aarch64-apple-darwin dylib — the uniffi metadata
# embedded in the library is ABI-independent; the resulting .kt file is identical.
cargo build -p jova-spike --release --features ffi --target aarch64-apple-darwin

uniffi-bindgen generate \
  target/aarch64-apple-darwin/release/libjova_spike.dylib \
  --language kotlin \
  --out-dir generated/kotlin \
  --no-format

echo "[ok] Android cross-compile + Kotlin bindings"
