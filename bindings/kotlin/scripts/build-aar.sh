#!/usr/bin/env bash
set -euo pipefail

# Run from repo root so cargo and cargo-ndk use the workspace target/ dir.
cd "$(dirname "$0")/../../.."

# shellcheck source=/dev/null
. "$HOME/.cargo/env" 2>/dev/null || true

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}"

# Locate Java on macOS when the shell hasn't sourced .zshrc (e.g. in the Claude
# Code harness). Prefer JAVA_HOME if already set; fall back to Android Studio's
# bundled JBR, then /usr/libexec/java_home.
if [ -z "${JAVA_HOME:-}" ]; then
  AS_JBR="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
  if [ -d "$AS_JBR" ]; then
    export JAVA_HOME="$AS_JBR"
  elif command -v /usr/libexec/java_home &>/dev/null; then
    export JAVA_HOME="$(/usr/libexec/java_home 2>/dev/null || true)"
  fi
fi
[ -n "${JAVA_HOME:-}" ] && export PATH="$JAVA_HOME/bin:$PATH"

# Cross-compile to all 4 Android ABIs. Output goes to the jniLibs source set.
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -o bindings/kotlin/jova-core/src/main/jniLibs \
  build -p jova-core-ffi --release

# Build a host-platform library so uniffi-bindgen can read metadata from it.
# uniffi-bindgen has to dlopen the library to extract the ABI definitions, and
# it can only dlopen libraries native to the host (Mach-O on macOS, ELF on Linux).
# uniffi metadata is ABI-independent, so the generated Kotlin is identical
# regardless of which host built the metadata source.
cargo build -p jova-core-ffi --release

case "$(uname -s)" in
  Darwin) HOST_LIB="target/release/libjova_core_ffi.dylib" ;;
  *)      HOST_LIB="target/release/libjova_core_ffi.so"    ;;
esac

uniffi-bindgen generate \
  --library "$HOST_LIB" \
  --language kotlin \
  --out-dir bindings/kotlin/jova-core/src/main/kotlin

# Sync the spec vectors into the JVM test resources.
mkdir -p bindings/kotlin/jova-core/src/test/resources
cp spec/test-vectors.json bindings/kotlin/jova-core/src/test/resources/test-vectors.json

# Assemble the AAR.
( cd bindings/kotlin && ./gradlew :jova-core:assembleRelease )
echo "AAR at bindings/kotlin/jova-core/build/outputs/aar/"
