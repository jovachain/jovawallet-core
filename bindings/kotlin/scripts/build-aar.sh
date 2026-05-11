#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Source cargo env so cargo-ndk and uniffi-bindgen are on PATH.
# shellcheck source=/dev/null
. "$HOME/.cargo/env"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}"

# Cross-compile to all 4 Android ABIs.
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -o jova-core/src/main/jniLibs \
  build -p jova-core-ffi --release --manifest-path ../../Cargo.toml

# Sync test resources.
cp ../../spec/test-vectors.json jova-core/src/test/resources/test-vectors.json

# Generate Kotlin bindings from the native macOS dylib, NOT the Android .so.
# Reason: uniffi-bindgen cannot dlopen ELF binaries on macOS host.
# The uniffi metadata is ABI-independent so the Kotlin output is correct for all Android ABIs.
uniffi-bindgen generate \
  --library ../../target/aarch64-apple-darwin/release/libjova_core_ffi.dylib \
  --language kotlin \
  --out-dir jova-core/src/main/kotlin

# Build the AAR.
./gradlew :jova-core:assembleRelease
echo "AAR at jova-core/build/outputs/aar/"
