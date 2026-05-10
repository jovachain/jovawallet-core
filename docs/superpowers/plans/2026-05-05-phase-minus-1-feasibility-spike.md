# Phase -1: Feasibility Spike Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Versions refreshed 2026-05-10 during env-prep session.** Crate and tool versions in this file were bumped to current latest stable on 2026-05-10 (see commit history). Most chain crates have had major version bumps since the original plan write (alloy 0.9 → 2.0, bdk_wallet 1.5 → 3.0, solana-* 2 → 3-4); breaking-API changes are likely. The spike's whole purpose is to discover and document these. If `lib.rs` snippets reference types that have moved/renamed in the new versions, fix the snippets and document in `docs/feasibility-report.md`.

**Goal:** Prove the toolchain (Rust + uniffi-rs + Swift XCFramework + Kotlin AAR + WASM) compiles end-to-end with the candidate chain dependencies (`bdk_wallet`, `alloy`, Anza's Solana split crates, `xrpl-rust`) on every target before any real code is written.

**Architecture:** A throwaway branch `spike/feasibility` containing a minimal `lib.rs` exporting one function (`ping() -> String`), wrapped through every binding, with all candidate chain crates listed in `Cargo.toml` so they actually link. End-state is a written report (`docs/feasibility-report.md`) saying which crates compile cleanly on which targets and what feature flags are needed.

**Tech Stack:** Rust 1.95.0 stable for the build (edition 2024 / MSRV 1.85), uniffi-rs 0.31.1, wasm-bindgen 0.2 + wasm-pack 0.14, cargo-ndk 4.1, Xcode 26+, Android NDK r29 (29.0.14206865), Java 21, Node 22+ (pnpm 10), GitHub Actions for CI. The 1.95.0 toolchain pin was confirmed current on 2026-05-10 (latest stable per `https://static.rust-lang.org/dist/channel-rust-stable.toml`); bump if a newer stable shipped between now and execution.

**Preconditions before starting:**
- Working directory: `/Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core`
- The working directory **is already a git repo** with `main` tracking `origin/main` at `git@github.com:jovachain/jovawallet-core.git` (the docs were pushed before Phase -1 began). Task 1 verifies this state and creates the spike branch.
- macOS host (for the Swift XCFramework build steps). If executing on Linux only, mark Swift tasks as "not validated locally" and rely on CI.

**Exit criteria for the whole phase:**
- A throwaway branch `spike/feasibility` builds clean for every target listed in Task 8.
- The Rust ping function round-trips through Swift, Kotlin, and JS hello-world consumers.
- `docs/feasibility-report.md` documents which chain crates compile on which targets, with concrete feature-flag findings.
- The user reviews the report and gives a go/no-go on Phase 0.

---

## Task 1: Verify repo state and create the spike branch

The git repo and the docs commit already exist (created during the docs-push step before Phase -1 began). This task verifies the state and creates the throwaway `spike/feasibility` branch.

**Files:** none new — `.gitignore`, `README.md`, `CLAUDE.md`, and `docs/` are already committed.

- [ ] **Step 1: Verify the repo state**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
git status
git log --oneline -5
git branch
git remote -v
```

Expected:
- Working tree clean.
- Currently on `main`.
- At least one commit (the docs initial commit).
- Remote `origin` configured pointing to `github.com:jovachain/jovawallet-core.git`.

If any of those don't hold, **stop and surface to the user**. The spike depends on starting from a known docs-pushed state.

- [ ] **Step 2: Verify `.gitignore` matches expectations**

```bash
cat .gitignore
```

Expected contents (already in place):

```
# Rust
target/
Cargo.lock.bak
**/*.rs.bk

# uniffi
generated/

# Swift
.build/
DerivedData/
*.xcframework
*.xcframework.zip

# Kotlin / Android
build/
.gradle/
local.properties
*.aar

# WASM / Node
node_modules/
pkg/
dist/
*.tgz

# OS
.DS_Store
Thumbs.db
```

If the contents differ, surface to the user before proceeding.

- [ ] **Step 3: Create the spike branch**

```bash
git checkout -b spike/feasibility
```

- [ ] **Step 4: Confirm branch state**

```bash
git status
git branch --show-current
```

Expected: clean tree on branch `spike/feasibility`.

The spike branch is throwaway — Phase 0 starts from `main`, which never received the spike commits. Spike work is preserved on the branch indefinitely as a historical record.

---

## Task 2: Set up Rust workspace with all candidate chain crates

**Files:**
- Create: `rust-toolchain.toml`
- Create: `Cargo.toml`
- Create: `crates/jova-spike/Cargo.toml`
- Create: `crates/jova-spike/src/lib.rs`

- [ ] **Step 1: Pin the Rust toolchain**

Create `rust-toolchain.toml`:

```toml
[toolchain]
# Pin to the latest stable as of May 2026. Spike confirms; bump if newer shipped.
channel = "1.95.0"
components = ["rustfmt", "clippy"]
targets = [
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    # x86_64-apple-ios is intentionally omitted: Apple deprecated the Intel iOS
    # simulator. Modern arm64 Macs run an arm64 simulator natively. Older Intel
    # Macs run iOS sim under Rosetta from the arm64 slice.
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "x86_64-linux-android",
    "i686-linux-android",
    "wasm32-unknown-unknown",
    "thumbv7em-none-eabihf",
]
```

- [ ] **Step 2: Create the workspace `Cargo.toml`**

```toml
[workspace]
resolver = "3"   # Rust 2024 edition default resolver
members = ["crates/jova-spike"]

[workspace.package]
edition = "2024"
rust-version = "1.85"   # MSRV: edition 2024 stabilized in 1.85; minimum bar for consumers.
license = "MIT"

[workspace.dependencies]
# Candidate chain crates — versions are current latest stable as of 2026-05-10.
# Several have had major-version bumps since the original plan write — feature
# flag names and type paths may have changed. The spike's job is to discover.
# If a feature listed here doesn't exist in the new major version, retry without
# it and document in feasibility-report.md.
alloy             = { version = "2.0",  default-features = false, features = ["consensus", "signer-local", "sol-types", "dyn-abi"] }   # was 0.9 in original plan
bdk_wallet        = { version = "3.0",  default-features = false }   # was 1.5 in original plan
bitcoin           = { version = "0.32", default-features = false, features = ["secp-recovery"] }   # was 0.33; 0.33.0-beta exists but no stable yet

# Solana: use Anza's split crates rather than the monolithic solana-sdk.
# The split tree is dramatically smaller and is WASM-viable.
solana-keypair    = { version = "3.1", default-features = false }   # was 2 in original plan
solana-pubkey     = { version = "4.2", default-features = false }   # was 2 in original plan
solana-signature  = { version = "3.4", default-features = false }   # was 2 in original plan
solana-transaction = { version = "4.1", default-features = false }   # was 2 in original plan
solana-message    = { version = "4.1", default-features = false }   # was 2 in original plan

xrpl              = { version = "0.1.2", default-features = false }   # original plan said 0.5 but only 0.1.2 exists on crates.io as of 2026-05-10. Spike must verify this is the intended crate; the xrpl-rust project may publish under a different name. If wrong crate, flag in feasibility-report.md.

# Crypto primitives — every chain crate above transitively depends on subsets of
# these; we list them explicitly because jova-core-primitives uses them directly.
secp256k1         = { version = "0.31", default-features = false, features = ["alloc", "lowmemory", "global-context"] }   # was 0.30; 0.32 is in beta
ed25519-dalek     = { version = "2.2",  default-features = false, features = ["alloc"] }   # was 2.1; 3.0 is in pre-release with active dev
bip39             = { version = "2.2",  default-features = false, features = ["english"] }
slip-10           = { version = "0.4",  default-features = false }   # The 2024 fork; crate name uses hyphen.
zeroize           = { version = "1.8",  default-features = false, features = ["alloc"] }

# FFI / WASM
uniffi            = { version = "0.31", features = ["build", "cli"] }   # was 0.29; macro must match installed uniffi-bindgen CLI version
wasm-bindgen      = { version = "0.2" }
serde-wasm-bindgen = { version = "0.6" }

# Util
serde             = { version = "1", default-features = false, features = ["derive", "alloc"] }
serde_json        = { version = "1", default-features = false, features = ["alloc"] }
thiserror         = "2"          # 2.0 stable since late 2024; replaces 1.x
hex               = { version = "0.4", default-features = false, features = ["alloc"] }

[profile.release]
codegen-units = 1
lto = "fat"
strip = "symbols"
panic = "abort"             # Smaller binary; signing SDK uses Result, not panic-recovery.
debug = "line-tables-only"  # Just enough for crash symbolication.

[profile.release.package."*"]
opt-level = 3
```

- [ ] **Step 3: Create the spike crate manifest**

`crates/jova-spike/Cargo.toml`:

```toml
[package]
name = "jova-spike"
version = "0.0.0-spike"
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["lib", "staticlib", "cdylib"]
name = "jova_spike"

[dependencies]
# Every candidate chain crate is OPTIONAL so we can isolate compile failures
# by toggling positive features.
alloy              = { workspace = true, optional = true }
bdk_wallet         = { workspace = true, optional = true }
bitcoin            = { workspace = true, optional = true }
solana-keypair     = { workspace = true, optional = true }
solana-transaction = { workspace = true, optional = true }
solana-message     = { workspace = true, optional = true }
solana-pubkey      = { workspace = true, optional = true }
solana-signature   = { workspace = true, optional = true }
xrpl               = { workspace = true, optional = true }

# Primitives are non-optional — they must link on every target unconditionally.
secp256k1.workspace     = true
ed25519-dalek.workspace = true
bip39.workspace         = true
slip-10.workspace       = true
zeroize.workspace       = true

uniffi       = { workspace = true, optional = true }
wasm-bindgen = { workspace = true, optional = true }

[features]
# All chains on by default; disable individually with --no-default-features
# and re-add the ones you want.
default   = ["chain-evm", "chain-btc", "chain-sol", "chain-xrp", "ffi"]
chain-evm = ["dep:alloy"]
chain-btc = ["dep:bdk_wallet", "dep:bitcoin"]
chain-sol = ["dep:solana-keypair", "dep:solana-transaction", "dep:solana-message", "dep:solana-pubkey", "dep:solana-signature"]
chain-xrp = ["dep:xrpl"]
ffi       = ["dep:uniffi"]
wasm      = ["dep:wasm-bindgen"]

[build-dependencies]
uniffi = { workspace = true, features = ["build"] }
```

- [ ] **Step 4: Create the spike `lib.rs`**

`crates/jova-spike/src/lib.rs`:

```rust
//! jova-spike — feasibility spike. Throwaway. Phase 0 starts from a clean slate.
//!
//! Goal: prove every target builds and links the candidate chain crates.

#[cfg_attr(feature = "ffi", uniffi::export)]
pub fn ping() -> String {
    "pong".to_string()
}

#[cfg_attr(feature = "ffi", uniffi::export)]
pub fn ping_chains() -> String {
    // Reference each enabled chain crate at least once so it isn't dead-stripped.
    #[cfg(feature = "chain-evm")]
    let _ = std::any::type_name::<alloy::consensus::TxEip1559>();

    #[cfg(feature = "chain-btc")]
    {
        let _ = std::any::type_name::<bitcoin::Address>();
        let _ = std::any::type_name::<bdk_wallet::Wallet>();
    }

    #[cfg(feature = "chain-sol")]
    {
        let _ = std::any::type_name::<solana_keypair::Keypair>();
        let _ = std::any::type_name::<solana_transaction::versioned::VersionedTransaction>();
    }

    #[cfg(feature = "chain-xrp")]
    let _ = std::any::type_name::<xrpl::core::keypairs::Seed>();

    "chains-linked".to_string()
}

#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ping_wasm() -> String {
    ping()
}
```

- [ ] **Step 5: Verify the host build**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
cargo build -p jova-spike
```

Expected: build succeeds. If a chain crate fails on the host, **stop and document the failure**. This is the cheapest place to discover incompatibility.

To isolate which chain is the culprit, retry with positive features:

```bash
# Baseline: primitives + ffi only.
cargo build -p jova-spike --no-default-features --features ffi

# Then add one chain at a time.
cargo build -p jova-spike --no-default-features --features ffi,chain-evm
cargo build -p jova-spike --no-default-features --features ffi,chain-btc
cargo build -p jova-spike --no-default-features --features ffi,chain-sol
cargo build -p jova-spike --no-default-features --features ffi,chain-xrp
```

Whichever feature flag turns the build red is the culprit. Record in the report.

- [ ] **Step 6: Run a smoke test**

Quick host-side check that `ping_chains` runs:

```bash
cargo test -p jova-spike --lib -- --nocapture
```

(No tests yet, but the compile-and-link is the test.)

- [ ] **Step 7: Commit**

```bash
git add rust-toolchain.toml Cargo.toml crates/jova-spike/
git commit -m "spike: rust workspace with all candidate chain crates linking"
```

---

## Task 3: Verify iOS XCFramework build

**Files:**
- Create: `spike/build-ios.sh`

We use uniffi-rs in **proc-macro-only mode** (`#[uniffi::export]` + `uniffi::setup_scaffolding!()`). No `build.rs`, no UDL file required. The `#[uniffi::export]` annotations in Task 2 are sufficient.

- [ ] **Step 1: Verify proc-macro-only uniffi setup is sufficient**

In `crates/jova-spike/src/lib.rs`, confirm `uniffi::setup_scaffolding!()` is present (it was added in Task 2, Step 4). If the macro is missing, add it now — it's the only thing needed besides the `#[uniffi::export]` annotations to wire up FFI scaffolding.

- [ ] **Step 2: Build the spike crate for iOS targets**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
for target in aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build -p jova-spike --release --target "$target" --features ffi
  echo "✅ $target"
done
```

Expected: each target succeeds. If a chain fails on an iOS target, retry with positive isolation:

```bash
# Find which chain is the culprit on this target.
cargo build -p jova-spike --target aarch64-apple-ios --release \
  --no-default-features --features ffi,chain-evm
cargo build -p jova-spike --target aarch64-apple-ios --release \
  --no-default-features --features ffi,chain-btc
# ... etc.
```

**Document each result for the report.**

- [ ] **Step 3: Generate the Swift bindings**

```bash
# Install uniffi CLI via the umbrella `uniffi` crate with the `cli` feature.
# In uniffi 0.30+ the CLI binary is gated behind this feature and lives inside
# the same crate the workspace depends on for the macro — so macro and CLI
# version match by construction. Installs `uniffi-bindgen` and `uniffi-bindgen-swift`.
cargo install uniffi --features cli --locked
uniffi-bindgen generate \
  --library target/aarch64-apple-darwin/release/libjova_spike.dylib \
  --language swift \
  --out-dir generated/swift
ls generated/swift/
```

Expected: `JovaSpike.swift`, `jova_spikeFFI.modulemap`, `jova_spikeFFI.h` exist.

> **Heads-up:** older docs and tutorials say `cargo install uniffi-bindgen-cli`. That crate name no longer exists on crates.io — use `cargo install uniffi --features cli` as above.

- [ ] **Step 4: Build the XCFramework**

Create `spike/build-ios.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

LIB_NAME=libjova_spike.a
HEADERS_DIR=generated/swift

# Mac slice is universal (arm64 + x86_64) via lipo.
mkdir -p target/mac-universal
lipo -create \
  target/aarch64-apple-darwin/release/$LIB_NAME \
  target/x86_64-apple-darwin/release/$LIB_NAME \
  -output target/mac-universal/$LIB_NAME

# Three slices: device, simulator (arm64-only), and macOS universal.
rm -rf generated/JovaSpikeFFI.xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/$LIB_NAME      -headers $HEADERS_DIR \
  -library target/aarch64-apple-ios-sim/release/$LIB_NAME  -headers $HEADERS_DIR \
  -library target/mac-universal/$LIB_NAME                  -headers $HEADERS_DIR \
  -output generated/JovaSpikeFFI.xcframework

echo "✅ XCFramework built at generated/JovaSpikeFFI.xcframework"
```

Run it:

```bash
chmod +x spike/build-ios.sh
./spike/build-ios.sh
```

Expected: `generated/JovaSpikeFFI.xcframework` exists with three slices: `ios-arm64`, `ios-arm64-simulator`, and `macos-arm64_x86_64`.

Note: the simulator slice is **arm64-only**. This is correct for 2026 — Apple has fully deprecated the Intel iOS simulator. Older Intel Macs run the iOS simulator under Rosetta from the arm64 slice.

- [ ] **Step 5: Smoke-test the XCFramework from a Swift script**

Create a temporary `spike/swift-smoke.swift`:

```swift
import Foundation
@_implementationOnly import JovaSpikeFFI

// uniffi-generated Swift API
print(ping())
print(pingChains())
```

We are not running this Swift script directly (would need a full Xcode project). Instead, we mark this as **CI-only validation** — the GitHub Actions runner with macOS will exercise it. Note in the report: "Swift smoke not run locally; CI proof required."

- [ ] **Step 6: Commit**

```bash
git add crates/jova-spike/src/lib.rs spike/build-ios.sh
git commit -m "spike: iOS XCFramework builds for iOS device, simulator, and macOS"
```

---

## Task 4: Verify Android AAR build via cargo-ndk

**Files:**
- Create: `spike/build-android.sh`

- [ ] **Step 1: Install cargo-ndk**

```bash
cargo install cargo-ndk --locked
```

(Current latest is 4.1.2 as of 2026-05-10 — major-version jump from the original plan's 3.5. CLI surface is largely the same; if any cargo-ndk command below fails, check the 4.x changelog at https://github.com/bbqsrc/cargo-ndk/releases.)

- [ ] **Step 2: Verify Android NDK is available**

```bash
echo "ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
ls "$ANDROID_NDK_HOME" || { echo "Set ANDROID_NDK_HOME to the NDK r29+ stable path (e.g., $HOME/Library/Android/sdk/ndk/29.0.14206865)"; exit 1; }
```

Expected: NDK directory listing. If not, the developer must install Android NDK r29 stable (29.0.14206865) per `docs/env-setup.md` Step 6 and export `ANDROID_NDK_HOME`. **Do not use r30-beta1 or any `-beta` / `-rc` build** — CI uses stable and beta NDKs can shift codegen between builds.

- [ ] **Step 3: Cross-compile to all Android ABIs**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -o generated/android/jniLibs \
  build -p jova-spike --release --features ffi
ls generated/android/jniLibs/
```

Expected: directories `arm64-v8a/`, `armeabi-v7a/`, `x86_64/`, `x86/` each containing `libjova_spike.so`.

If a chain crate fails on a specific Android ABI (the historical pain point was `armeabi-v7a` 32-bit on the monolithic `solana-sdk`; the Anza split crates should be cleaner), retry with positive isolation:

```bash
cargo ndk -t armeabi-v7a -o /tmp/probe build -p jova-spike --release \
  --no-default-features --features ffi,chain-evm
# ... etc., one chain at a time, to find the culprit.
```

Document the result.

- [ ] **Step 4: Generate Kotlin bindings**

```bash
uniffi-bindgen generate \
  --library target/aarch64-linux-android/release/libjova_spike.so \
  --language kotlin \
  --out-dir generated/kotlin
ls generated/kotlin/
```

Expected: `uniffi/jova_spike/jova_spike.kt` exists.

(Binary is `uniffi-bindgen`, not `uniffi-bindgen-cli` — installed in Task 3 Step 3 via `cargo install uniffi --features cli --locked`.)

- [ ] **Step 5: Smoke-test through a tiny Gradle project**

Mark as **CI-only validation** — JVM smoke runs in `ci-bindings-kotlin.yml` once that exists. Locally we just confirmed the .so files cross-compile and bindings generate.

- [ ] **Step 6: Commit**

```bash
git add generated/.gitignore 2>/dev/null || true
git add spike/build-android.sh 2>/dev/null || true
echo "generated/" >> .gitignore
git add .gitignore
git commit -m "spike: Android AAR cross-compiles for all 4 ABIs; Kotlin bindings generate"
```

---

## Task 5: Verify WASM build via wasm-bindgen

**Files:**
- Create: `crates/jova-spike-wasm/Cargo.toml`
- Create: `crates/jova-spike-wasm/src/lib.rs`
- Create: `spike/build-wasm.sh`

- [ ] **Step 1: Add a separate WASM crate**

The same `jova-spike` crate would conflict because uniffi targets aren't WASM-friendly. We use a sibling crate.

`crates/jova-spike-wasm/Cargo.toml`:

```toml
[package]
name = "jova-spike-wasm"
version = "0.0.0-spike"
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
alloy = { workspace = true }
bitcoin = { workspace = true }
bdk_wallet = { workspace = true, optional = true }   # optional in case it fights WASM
solana-keypair     = { workspace = true, optional = true }
solana-transaction = { workspace = true, optional = true }
solana-message     = { workspace = true, optional = true }
solana-pubkey      = { workspace = true, optional = true }
solana-signature   = { workspace = true, optional = true }
xrpl = { workspace = true, optional = true }
secp256k1 = { workspace = true }
ed25519-dalek = { workspace = true }
bip39 = { workspace = true }
slip-10 = { workspace = true }
zeroize = { workspace = true }
wasm-bindgen.workspace = true
getrandom = { version = "0.4", features = ["wasm_js"] }   # WASM needs explicit RNG; getrandom uses wasm_js feature flag (0.4 current as of 2026-05-10; was 0.3 in original plan)

[features]
default = []
chain-bdk = ["dep:bdk_wallet"]
chain-sol = ["dep:solana-keypair", "dep:solana-transaction", "dep:solana-message", "dep:solana-pubkey", "dep:solana-signature"]
chain-xrp = ["dep:xrpl"]
all-chains = ["chain-bdk", "chain-sol", "chain-xrp"]
```

- [ ] **Step 2: Add the spike crate to the workspace**

Modify `Cargo.toml` (workspace root):

```toml
[workspace]
resolver = "3"
members = ["crates/jova-spike", "crates/jova-spike-wasm"]
```

- [ ] **Step 3: Create the WASM `lib.rs`**

`crates/jova-spike-wasm/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn ping_wasm() -> String {
    "pong-wasm".to_string()
}

#[wasm_bindgen]
pub fn ping_chains_wasm() -> String {
    let _ = std::any::type_name::<alloy::consensus::TxEip1559>();
    let _ = std::any::type_name::<bitcoin::Address>();

    #[cfg(feature = "chain-bdk")]
    let _ = std::any::type_name::<bdk_wallet::Wallet>();

    #[cfg(feature = "chain-sol")]
    {
        let _ = std::any::type_name::<solana_keypair::Keypair>();
        let _ = std::any::type_name::<solana_transaction::versioned::VersionedTransaction>();
    }

    #[cfg(feature = "chain-xrp")]
    let _ = std::any::type_name::<xrpl::core::keypairs::Seed>();

    "chains-linked-wasm".to_string()
}
```

- [ ] **Step 4: Build the baseline WASM (no chains)**

```bash
cargo install wasm-pack --locked
cd crates/jova-spike-wasm
wasm-pack build --release --target web --out-dir ../../generated/wasm-baseline
ls ../../generated/wasm-baseline/
```

(Current latest wasm-pack is 0.14.0 as of 2026-05-10; was 0.13 in original plan.)

Expected: `pkg/jova_spike_wasm.js`, `pkg/jova_spike_wasm_bg.wasm`, etc.

- [ ] **Step 5: Try building with each chain feature one at a time**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
for feat in chain-bdk chain-sol chain-xrp; do
  echo "==== Trying WASM build with feature: $feat ===="
  (cd crates/jova-spike-wasm && \
    wasm-pack build --release --target web \
      --out-dir "../../generated/wasm-$feat" \
      -- --features "$feat") \
    && echo "✅ $feat compiles to WASM" \
    || echo "❌ $feat FAILS WASM — document"
done
```

This is the dragons-or-not test. **Document the result for every feature in the report.** Solana especially: even with the Anza split crates the bundle may need feature-trimming on WASM (e.g., disabling default features that pull in `tokio` or `std::os`). If trimming isn't enough, the fallback is to ship Solana native-only and exclude it from the WASM build.

- [ ] **Step 6: Try the full all-chains WASM build**

```bash
(cd crates/jova-spike-wasm && \
  wasm-pack build --release --target web --out-dir ../../generated/wasm-all \
    -- --features all-chains) \
  && echo "✅ all-chains compile" \
  || echo "❌ all-chains FAIL — see per-feature log"
```

- [ ] **Step 7: Smoke-test the baseline WASM in Node**

Create `spike/wasm-smoke.mjs`:

```javascript
import init, { ping_wasm } from '../generated/wasm-baseline/jova_spike_wasm.js';
await init();
const result = ping_wasm();
console.log(result);
if (result !== 'pong-wasm') {
    console.error('FAIL: expected pong-wasm');
    process.exit(1);
}
console.log('✅ WASM round-trip works');
```

Run:

```bash
node spike/wasm-smoke.mjs
```

Expected: `pong-wasm` printed. (Note: requires Node 20+ with WASM ESM support.)

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml crates/jova-spike-wasm/ spike/build-wasm.sh spike/wasm-smoke.mjs
git commit -m "spike: WASM target compiles; per-chain feature-flag results documented"
```

---

## Task 6: Verify no_std build for the primitives stack

**Files:** None new (uses workspace as-is).

- [ ] **Step 1: Add a tiny no_std test crate**

Modify `Cargo.toml` (workspace root):

```toml
[workspace]
resolver = "2"
members = ["crates/jova-spike", "crates/jova-spike-wasm", "crates/jova-spike-nostd"]
```

Create `crates/jova-spike-nostd/Cargo.toml`:

```toml
[package]
name = "jova-spike-nostd"
version = "0.0.0-spike"
edition.workspace = true
license.workspace = true

[dependencies]
secp256k1     = { workspace = true, default-features = false, features = ["alloc", "lowmemory"] }
ed25519-dalek = { workspace = true, default-features = false, features = ["alloc"] }
bip39         = { workspace = true, default-features = false, features = ["english"] }
slip-10       = { workspace = true, default-features = false }
zeroize       = { workspace = true, default-features = false }

[lib]
name = "jova_spike_nostd"
crate-type = ["lib"]
```

Create `crates/jova-spike-nostd/src/lib.rs`:

```rust
#![no_std]
extern crate alloc;

pub fn smoke() -> &'static str {
    let _ = core::any::type_name::<bip39::Mnemonic>();
    let _ = core::any::type_name::<secp256k1::SecretKey>();
    let _ = core::any::type_name::<ed25519_dalek::SigningKey>();
    "nostd-ok"
}
```

- [ ] **Step 2: Build for thumbv7em**

```bash
cargo build -p jova-spike-nostd --target thumbv7em-none-eabihf --release
```

Expected: build succeeds. If a primitive accidentally pulls in `std`, fix the feature flags or document the gap (e.g., a primitive crate needs `default-features = false` we missed).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml crates/jova-spike-nostd/
git commit -m "spike: jova-core-primitives stack builds for thumbv7em-none-eabihf"
```

---

## Task 7: Add a CI workflow that exercises the spike on push

**Files:**
- Create: `.github/workflows/spike-feasibility.yml`

- [ ] **Step 1: Create the workflow**

`.github/workflows/spike-feasibility.yml`:

```yaml
name: spike-feasibility
on:
  push:
    branches: [spike/feasibility]
  workflow_dispatch:

jobs:
  rust-host:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace --release

  ios-build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios,aarch64-apple-ios-sim,aarch64-apple-darwin,x86_64-apple-darwin
      - run: cargo install uniffi --features cli --locked
      - run: |
          for target in aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin; do
            cargo build -p jova-spike --release --target "$target" --features ffi
          done
      - run: ./spike/build-ios.sh

  android-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android,armv7-linux-androideabi,x86_64-linux-android,i686-linux-android
      - uses: nttld/setup-ndk@v2
        id: ndk
        with:
          ndk-version: r29   # latest stable per github.com/android/ndk/releases as of 2026-05-10
      - run: cargo install cargo-ndk --locked
      - run: cargo install uniffi --features cli --locked
      - env:
          ANDROID_NDK_HOME: ${{ steps.ndk.outputs.ndk-path }}
        run: |
          cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 \
            -o generated/android/jniLibs \
            build -p jova-spike --release --features ffi

  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo install wasm-pack --locked
      - run: |
          cd crates/jova-spike-wasm
          wasm-pack build --release --target web --out-dir ../../generated/wasm-baseline
      - run: |
          for feat in chain-bdk chain-sol chain-xrp; do
            # Normalize hyphens → underscores; uppercase. Hyphens are invalid
            # in shell variable names, so writing PASS_chain-bdk fails silently.
            name="PASS_$(echo "$feat" | tr 'a-z-' 'A-Z_')"
            if (cd crates/jova-spike-wasm && \
                wasm-pack build --release --target web \
                  --out-dir "../../generated/wasm-$feat" \
                  -- --features "$feat"); then
              echo "$name=true"  >> $GITHUB_ENV
            else
              echo "$name=false" >> $GITHUB_ENV
            fi
          done
      - run: |
          echo "WASM feature-flag results:"
          echo "  chain-bdk: ${PASS_CHAIN_BDK:-unknown}"
          echo "  chain-sol: ${PASS_CHAIN_SOL:-unknown}"
          echo "  chain-xrp: ${PASS_CHAIN_XRP:-unknown}"

  no-std-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: thumbv7em-none-eabihf
      - run: cargo build -p jova-spike-nostd --target thumbv7em-none-eabihf --release
```

- [ ] **Step 2: Commit and push the spike branch**

```bash
git add .github/workflows/spike-feasibility.yml
git commit -m "spike: CI exercises every target on every push"
```

If a remote `origin` is configured (it isn't required for the spike), push to GitHub:

```bash
git push -u origin spike/feasibility 2>/dev/null || echo "No remote configured yet; CI runs locally only"
```

---

## Task 8: Write the feasibility report

**Files:**
- Create: `docs/feasibility-report.md`

- [ ] **Step 1: Document every result**

Create `docs/feasibility-report.md`:

```markdown
# Feasibility Report — Phase -1

**Date:** YYYY-MM-DD
**Branch:** `spike/feasibility`
**Commit SHA:** <fill in>

## Targets exercised

| Target | Spike crate | Result | Notes |
|---|---|---|---|
| `aarch64-apple-ios` | jova-spike | ✅ / ❌ | |
| `aarch64-apple-ios-sim` | jova-spike | ✅ / ❌ | |
| `aarch64-apple-darwin` | jova-spike | ✅ / ❌ | |
| `x86_64-apple-darwin` | jova-spike | ✅ / ❌ | |
| `aarch64-linux-android` | jova-spike | ✅ / ❌ | |
| `armv7-linux-androideabi` | jova-spike | ✅ / ❌ | |
| `x86_64-linux-android` | jova-spike | ✅ / ❌ | |
| `i686-linux-android` | jova-spike | ✅ / ❌ | |
| `wasm32-unknown-unknown` baseline | jova-spike-wasm (no chains) | ✅ / ❌ | |
| `wasm32-unknown-unknown` + bdk_wallet | jova-spike-wasm | ✅ / ❌ | |
| `wasm32-unknown-unknown` + Solana split crates | jova-spike-wasm | ✅ / ❌ | Bundle: `solana-keypair` + `solana-transaction` + `solana-message` + `solana-pubkey` + `solana-signature` |
| `wasm32-unknown-unknown` + xrpl-rust | jova-spike-wasm | ✅ / ❌ | |
| `thumbv7em-none-eabihf` | jova-spike-nostd | ✅ / ❌ | |

## Per-crate findings

### `bdk_wallet`
- Native (iOS, Android, Linux, macOS): ✅ / ❌
- WASM: ✅ / ❌
- Notes:

### `alloy`
- Native: ✅ / ❌
- WASM: ✅ / ❌
- Notes:

### Anza Solana split crates
Tested as a bundle (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`). If one of the five fails individually, document which.
- Native: ✅ / ❌
- WASM: ✅ / ❌
- Notes (per-crate detail if any single crate misbehaves; otherwise treat as one bundle):

### `xrpl-rust` (crate name `xrpl`)
- Native: ✅ / ❌
- WASM: ✅ / ❌
- Notes:

### `secp256k1` / `ed25519-dalek` / `bip39` / `slip-10`
- Native: ✅
- WASM: ✅
- no_std: ✅
- Notes:

## Decisions

For each chain crate that fails on a target, record the resolution:
- **Replace with X** — alternative crate (e.g., for Solana, our own ed25519+bincode-shaped signing if all five Anza crates fight WASM).
- **Feature-flag off** — this chain ships native-only; WASM excludes it.
- **Defer to Phase Y** — work around for now, fix later.

## Recommended Phase 0 dependency configuration

Based on findings above, the workspace `Cargo.toml` for Phase 0 should declare:

```toml
[workspace.dependencies]
# (write the actual decided versions and feature flags)
```

## Open questions

- (List anything that still needs research before Phase 0 begins.)

## Go / no-go for Phase 0

- ☐ All native targets compile every chain.
- ☐ WASM compiles at least baseline (no chains).
- ☐ Per-chain WASM situation is documented (pass / replace / defer).
- ☐ no_std primitives build clean.

If all four are checked: **GO** for Phase 0.
If any are unchecked: report blockers; the user decides whether to proceed, swap dependencies, or extend the spike.
```

- [ ] **Step 2: Fill in the report**

Run through every CI job's output (or local results from Tasks 2–6) and fill in each row. Be honest: if a target fails, write the actual error message in the Notes column.

- [ ] **Step 3: Commit the report**

```bash
git add docs/feasibility-report.md
git commit -m "spike: feasibility report — go/no-go for Phase 0"
```

- [ ] **Step 4: Hand off to user for review**

The agent should stop here and surface the report to the user. The user reads `docs/feasibility-report.md` and decides:

- **Go:** proceed to Phase 0 plan execution. Merge `spike/feasibility` to `main` (or close it; the spike code is throwaway and Phase 0 starts from a clean slate).
- **No-go:** revisit dependency choices. The user may ask the agent to extend the spike (e.g., swap a Solana split crate for in-house signing if WASM coverage is the issue).

---

## Self-review checklist for this plan

- [ ] Every task has exact file paths.
- [ ] Every code step has the actual code, not "add appropriate logic."
- [ ] Every command is exact and copy-pasteable.
- [ ] Expected outputs are stated.
- [ ] Failure modes are addressed (e.g., "if cargo-ndk fails, document").
- [ ] Exit criteria are clear.
- [ ] No "TODO" or "TBD" or "etc." in any task.

---

## What this plan does NOT do

- Does not produce production code. Everything is throwaway.
- Does not produce tests beyond compile-and-run smoke. Vector tests come in Phase 1.
- Does not publish anything to any registry.
- Does not remove the throwaway crates at the end — they live in the `spike/feasibility` branch indefinitely as historical record. Phase 0 starts from `main`, which never received the spike commits.

---

## Estimated time

3–5 days for a senior engineer comfortable with the toolchain. Add 50% if Rust+iOS+Android cross-compilation is new territory. The single biggest time sink is usually getting the Android NDK + cargo-ndk path correct.
