# jovawallet-core — common project tasks. Run `just` to list.

default:
    @just --list

# Build everything in release mode.
build:
    cargo build --workspace --release

# Run all Rust tests on the host.
test:
    cargo test --workspace --locked
    cargo run -p jova-verify-spec

# Lint: fmt + clippy (deny warnings).
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

# Verify primitives crate is no_std-clean.
no-std-check:
    cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --no-default-features

# Build the iOS XCFramework. Requires macOS host.
build-ios:
    bindings/swift/scripts/build-xcframework.sh

# Build the Android AAR. Requires NDK r27c+.
build-android:
    bindings/kotlin/scripts/build-aar.sh

# Build the WASM npm package.
build-wasm:
    bindings/wasm/scripts/build-wasm.sh

# Run every binding's test suite. Heavy; only on macOS host.
test-bindings:
    just build-ios && (cd bindings/swift && swift test)
    just build-android && (cd bindings/kotlin && ./gradlew :jova-core:test)
    just build-wasm && (cd bindings/wasm && pnpm install && pnpm test)

# Audit dependencies.
audit:
    cargo audit
    cargo deny check
    cargo machete    # detect unused deps

# Run cargo-fuzz on every target for 60 seconds.
fuzz:
    for t in fuzz_eip1559_decode fuzz_eip712_typed fuzz_address_parse; do \
        cargo +nightly fuzz run "$t" -- -max_total_time=60 ; \
    done
