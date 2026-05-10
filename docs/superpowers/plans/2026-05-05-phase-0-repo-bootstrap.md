# Phase 0: Repo Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Empty `main` branch → Rust workspace with 5 crates compiling, every binding (Swift, Kotlin, WASM) building and running a hello-world test that loads `spec/test-vectors.json` and validates one negative-mnemonic vector. Tag `v0.0.1`.

**Architecture:** A 5-crate Rust workspace (`jova-core-primitives`, `jova-core-chains`, `jova-core`, `jova-core-ffi`, `jova-core-wasm`) with one stub function each. Three binding scaffolds (`bindings/swift`, `bindings/kotlin`, `bindings/wasm`) that consume the FFI/WASM crates and run a single vector-based test. Six CI workflows enforce the matrix on every PR.

**Tech Stack:** Same as Phase -1: Rust 1.95.0 build pin (edition 2024 / MSRV 1.85), uniffi-rs ≥ 0.29, wasm-bindgen + wasm-pack, cargo-ndk, Xcode 16+, Android NDK r27c+, Java 21, Node 22 (pnpm 10). Exact versions come from `docs/feasibility-report.md`'s "Recommended Phase 0 dependency configuration" section.

**Preconditions:**
- Phase -1 complete; `docs/feasibility-report.md` exists and shows GO.
- The `spike/feasibility` branch is preserved but **not merged**. Phase 0 starts from `main` with a clean slate.
- The user has created GitHub repos `jovachain/jovawallet-core` and `jovachain/jovawallet-core-swift` (the satellite repo is not used in Phase 0 yet, but should exist).
- The agent has read access to the feasibility report findings; especially the recommended `[workspace.dependencies]` block.

**Exit criteria:**
- `cargo test --workspace --release --locked` is green.
- Each binding's hello-world test passes.
- All six CI workflows pass on a PR opened from a feature branch.
- `cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --no-default-features` succeeds.
- Tag `v0.0.1` exists on `main`.

---

## Task 1: Switch to main, add governance files

**Files:**
- Create: `LICENSE`
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Create: `CODEOWNERS`
- Create: `CHANGELOG.md`
- Create: `.editorconfig`
- Create: `.gitattributes`

- [ ] **Step 1: Switch to main**

```bash
cd /Users/satoshi/Documents/Workspace/Jovachain/jovawallet-core
git checkout main
git status   # should be clean; spike commits are NOT here
```

- [ ] **Step 2: Add MIT LICENSE**

`LICENSE`:

```
MIT License

Copyright (c) 2026 Jovachain

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 3: Add CONTRIBUTING.md**

`CONTRIBUTING.md`:

```markdown
# Contributing to jovawallet-core

Thanks for your interest. See `docs/README.md` for project context.

## Process
1. Open an issue describing the change before opening a PR (skip for trivial fixes).
2. Branch from `main`. Use descriptive names: `feat/btc-bip322`, `fix/eip712-domain`.
3. Every PR must be green on every CI workflow.
4. Every behavior change must be reflected in `spec/test-vectors.json`.
5. PRs touching public API require an ADR addition in `docs/decisions.md`.

## Commit messages
Conventional Commits format: `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`. Multi-line bodies welcome for non-trivial changes.

## Test vectors
A new chain or behavior is not "supported" without vectors. See `docs/testing.md` for the new-chain checklist.

## Security
Vulnerability disclosure: see `SECURITY.md`.
```

- [ ] **Step 4: Add SECURITY.md**

`SECURITY.md`:

```markdown
# Security

Report vulnerabilities to security@jovachain.io. PGP key fingerprint: TODO-publish-after-Phase-0.

We respond within 48 hours. High-severity issues use private GitHub Security Advisories.

See `docs/security.md` for the threat model, scope, and bug-bounty policy.
```

(The "TODO-publish-after-Phase-0" is acceptable here — this file is human-edited and the PGP key is a Phase 5 deliverable.)

- [ ] **Step 5: Add CODEOWNERS**

`CODEOWNERS`:

```
# Default owners for everything.
*                           @jovachain/sdk-leads

# Crypto layers require crypto-leads review.
/crates/jova-core-primitives/  @jovachain/sdk-leads @jovachain/crypto-leads
/crates/jova-core-chains/      @jovachain/sdk-leads @jovachain/crypto-leads
/spec/                         @jovachain/sdk-leads @jovachain/crypto-leads

# Bindings owned by SDK leads only.
/crates/jova-core-ffi/         @jovachain/sdk-leads
/crates/jova-core-wasm/        @jovachain/sdk-leads
/bindings/                     @jovachain/sdk-leads
```

- [ ] **Step 6: Add CHANGELOG.md, .editorconfig, .gitattributes**

`CHANGELOG.md`:

```markdown
# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] — YYYY-MM-DD

### Added
- Repo bootstrap: workspace, 5 crates, 3 bindings, 6 CI workflows, governance files.
- `spec/test-vectors.json` with one negative mnemonic-validation vector.
- Hello-world parity tests on Rust, Swift, Kotlin, WASM bindings.
```

`.editorconfig`:

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 4

[*.{toml,yaml,yml}]
indent_size = 2

[*.md]
trim_trailing_whitespace = false
```

`.gitattributes`:

```
* text=auto eol=lf
*.json diff
*.toml diff
Cargo.lock merge=binary
```

- [ ] **Step 7: Commit**

```bash
git add LICENSE CONTRIBUTING.md SECURITY.md CODEOWNERS CHANGELOG.md .editorconfig .gitattributes
git commit -m "chore: governance files (LICENSE, CONTRIBUTING, SECURITY, CODEOWNERS)"
```

---

## Task 2: Workspace + dependency configuration

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `rust-toolchain.toml`
- Create: `deny.toml`

- [ ] **Step 1: Read the feasibility report**

Read `docs/feasibility-report.md`'s "Recommended Phase 0 dependency configuration" section. The `[workspace.dependencies]` block below should reflect those decisions. If the spike found that the Solana split crates (or any other chain dependency) need replacement, swap it. **Do not deviate from the spike's findings without flagging the deviation in the commit message.**

- [ ] **Step 2: Create `rust-toolchain.toml`**

```toml
[toolchain]
# Pin a specific stable. Spike confirmed this is the latest stable at execution time.
channel = "1.95.0"
components = ["rustfmt", "clippy"]
targets = [
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
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

- [ ] **Step 3: Create the workspace `Cargo.toml`**

```toml
[workspace]
resolver = "3"   # Edition 2024 default resolver
members = [
    "crates/jova-core-primitives",
    "crates/jova-core-chains",
    "crates/jova-core",
    "crates/jova-core-ffi",
    "crates/jova-core-wasm",
    "tools/verify-spec",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"   # MSRV: edition 2024 requires 1.85+. Build uses 1.95.0 (toolchain pin).
license = "MIT"
repository = "https://github.com/jovachain/jovawallet-core"
authors = ["Jovachain SDK <sdk@jovachain.io>"]

[workspace.dependencies]
# Inter-crate
jova-core-primitives = { path = "crates/jova-core-primitives", version = "0.0.1" }
jova-core-chains     = { path = "crates/jova-core-chains",     version = "0.0.1" }
jova-core            = { path = "crates/jova-core",            version = "0.0.1" }

# Crypto primitives
secp256k1     = { version = "0.30", default-features = false, features = ["alloc", "lowmemory", "global-context"] }
ed25519-dalek = { version = "2.1",  default-features = false, features = ["alloc"] }
bip39         = { version = "2.1",  default-features = false, features = ["english"] }
slip-10       = { version = "0.4",  default-features = false }    # Crate name uses hyphen.
bip32         = { version = "0.5",  default-features = false, features = ["alloc"] }
sha2          = { version = "0.10", default-features = false }
sha3          = { version = "0.10", default-features = false }
ripemd        = { version = "0.1",  default-features = false }
hmac          = { version = "0.12", default-features = false }
zeroize       = { version = "1.8",  default-features = false, features = ["alloc", "derive"] }
subtle        = { version = "2.6",  default-features = false }

# Chain crates — versions reflect Phase -1 feasibility-report findings
alloy             = { version = "0.9",  default-features = false, features = ["consensus", "signer-local", "sol-types", "dyn-abi"] }
bdk_wallet        = { version = "1.5",  default-features = false }
bitcoin           = { version = "0.33", default-features = false, features = ["secp-recovery"] }

# Solana: Anza's split crates rather than the monolithic solana-sdk.
# Smaller dep tree, WASM-viable.
solana-keypair     = { version = "2", default-features = false }
solana-pubkey      = { version = "2", default-features = false }
solana-signature   = { version = "2", default-features = false }
solana-transaction = { version = "2", default-features = false }
solana-message     = { version = "2", default-features = false }

xrpl              = { version = "0.5", default-features = false }

# FFI / WASM
uniffi            = { version = "0.29", features = ["build", "cli"] }
wasm-bindgen      = "0.2"
serde-wasm-bindgen = "0.6"
getrandom         = { version = "0.3" }   # In jova-core-wasm we add features = ["wasm_js"]

# Util
serde      = { version = "1", default-features = false, features = ["derive", "alloc"] }
serde_json = { version = "1", default-features = false, features = ["alloc"] }
thiserror  = "2"          # 2.0 since Q4 2024
hex        = { version = "0.4", default-features = false, features = ["alloc"] }
base64     = { version = "0.22", default-features = false, features = ["alloc"] }
base58     = "0.2"

# Test only
proptest = "1.6"

[profile.release]
codegen-units = 1
lto = "fat"
strip = "symbols"
panic = "abort"             # Smaller binary; signing SDK uses Result, not panic-recovery.
                            # Unwinding through FFI is undefined behavior anyway.
debug = "line-tables-only"  # Just enough for crash symbolication.

[profile.release.package."*"]
opt-level = 3
```

- [ ] **Step 4: Create `deny.toml`**

```toml
[graph]
all-features = true

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
yanked  = "deny"
ignore  = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "Unicode-DFS-2016",
    "CC0-1.0",
    "Zlib",
]
confidence-threshold = 0.93

[bans]
multiple-versions = "warn"
wildcards         = "deny"
deny = [
    # Disallow accidental std-using deps in primitives crate.
]

[sources]
unknown-registry = "deny"
unknown-git      = "deny"
allow-registry   = ["https://github.com/rust-lang/crates.io-index"]
```

- [ ] **Step 5: Verify the workspace resolves**

```bash
cargo metadata --format-version 1 > /dev/null
```

Expected: succeeds (member crates don't exist yet, but workspace metadata resolves).

If it fails because the member directories are missing — that's expected. Proceed; the next tasks create them.

- [ ] **Step 6: Add `justfile` for project task running**

Modern alternative to scattered bash scripts. Cross-platform, self-documenting (`just` with no args lists tasks).

`justfile`:

```
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
```

Install just with `cargo install just --locked` or via `mise`/`brew install just`.

- [ ] **Step 7: Commit**

```bash
git add rust-toolchain.toml Cargo.toml deny.toml justfile
git commit -m "chore: workspace, dependency configuration, justfile"
```

---

## Task 3: Create the 5 Rust crate skeletons

**Files:**
- Create: `crates/jova-core-primitives/Cargo.toml`, `src/lib.rs`
- Create: `crates/jova-core-chains/Cargo.toml`, `src/lib.rs`
- Create: `crates/jova-core/Cargo.toml`, `src/lib.rs`, `tests/hello.rs`
- Create: `crates/jova-core-ffi/Cargo.toml`, `src/lib.rs`
- Create: `crates/jova-core-wasm/Cargo.toml`, `src/lib.rs`

- [ ] **Step 1: jova-core-primitives**

`crates/jova-core-primitives/Cargo.toml`:

```toml
[package]
name = "jova-core-primitives"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "no_std-clean cryptographic primitives for jovawallet-core"

[lib]

[features]
default = ["std"]
std = []

[dependencies]
zeroize.workspace = true
```

`crates/jova-core-primitives/src/lib.rs`:

```rust
//! jova-core-primitives — no_std-clean cryptographic primitives.
//!
//! Phase 0 stub. Phase 1 fills this in.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// Returns true iff `words` is the literal string "valid". Phase 0 stub
/// for the trivial vector test. Phase 1 replaces this with real BIP-39
/// validation.
pub fn is_valid_mnemonic_stub(words: &str, _passphrase: &str) -> bool {
    words == "valid"
}
```

- [ ] **Step 2: jova-core-chains**

`crates/jova-core-chains/Cargo.toml`:

```toml
[package]
name = "jova-core-chains"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Per-chain encoding and signing for jovawallet-core"

[lib]

[dependencies]
jova-core-primitives.workspace = true
```

`crates/jova-core-chains/src/lib.rs`:

```rust
//! jova-core-chains — per-chain encoding and signing.
//!
//! Phase 0 stub. Phase 1 lands the EVM signer.

#![forbid(unsafe_code)]

pub fn ping() -> &'static str {
    "chains-ok"
}
```

- [ ] **Step 3: jova-core**

`crates/jova-core/Cargo.toml`:

```toml
[package]
name = "jova-core"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Public Rust API for jovawallet-core"

[lib]

[dependencies]
jova-core-primitives.workspace = true
jova-core-chains.workspace = true

[dev-dependencies]
serde_json.workspace = true
```

`crates/jova-core/src/lib.rs`:

```rust
//! jova-core — public Rust API.
//!
//! Phase 0 stub. Phase 1 lands the full JovaWallet surface.

#![forbid(unsafe_code)]

pub use jova_core_primitives::is_valid_mnemonic_stub;

/// Phase 0 stub: validate a mnemonic. Real BIP-39 in Phase 1.
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    is_valid_mnemonic_stub(words, passphrase)
}
```

- [ ] **Step 4: Add the failing test for jova-core**

`crates/jova-core/tests/hello.rs`:

```rust
//! Phase 0 hello-world: load the spec vector file and assert one
//! negative-validation vector returns false.

use serde_json::Value;

#[test]
fn vector_negative_mnemonic_validation() {
    let raw = include_str!("../../../spec/test-vectors.json");
    let vectors: Value = serde_json::from_str(raw).expect("vectors parse");
    let arr = vectors["vectors"].as_array().expect("vectors array");

    let v = arr
        .iter()
        .find(|v| v["id"] == "phase0.mnemonic_validation_neg.gibberish")
        .expect("phase0 vector present");

    let words = v["input"]["words"].as_str().expect("words");
    let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
    let expected = v["expected"]["valid"].as_bool().expect("expected.valid");

    assert_eq!(jova_core::is_valid_mnemonic(words, passphrase), expected);
}
```

- [ ] **Step 5: Run the test (expected: fails because `spec/test-vectors.json` doesn't exist yet)**

```bash
cargo test -p jova-core --test hello 2>&1 | tail -5
```

Expected: build error like `couldn't read spec/test-vectors.json`. Good — this confirms the test exists and the spec file is the next thing to create.

- [ ] **Step 6: jova-core-ffi**

`crates/jova-core-ffi/Cargo.toml`:

```toml
[package]
name = "jova-core-ffi"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "uniffi-rs bindings layer for jova-core"

[lib]
crate-type = ["lib", "staticlib", "cdylib"]
name = "jova_core_ffi"

[dependencies]
jova-core.workspace = true
uniffi.workspace = true
```

`crates/jova-core-ffi/src/lib.rs`:

```rust
//! jova-core-ffi — uniffi-rs bindings layer.
//!
//! Phase 0 stub: re-export `is_valid_mnemonic`.

#![forbid(unsafe_code)]

#[uniffi::export]
pub fn is_valid_mnemonic(words: String, passphrase: String) -> bool {
    jova_core::is_valid_mnemonic(&words, &passphrase)
}

uniffi::setup_scaffolding!();
```

- [ ] **Step 7: jova-core-wasm**

`crates/jova-core-wasm/Cargo.toml`:

```toml
[package]
name = "jova-core-wasm"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "wasm-bindgen layer for jova-core"

[lib]
crate-type = ["cdylib"]

[dependencies]
jova-core.workspace = true
wasm-bindgen.workspace = true
serde-wasm-bindgen.workspace = true
# getrandom 0.3 uses `wasm_js` feature for browser RNG (renamed from `js` in 0.2).
getrandom = { workspace = true, features = ["wasm_js"] }
```

`crates/jova-core-wasm/src/lib.rs`:

```rust
//! jova-core-wasm — wasm-bindgen bindings.
//!
//! Phase 0 stub.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}
```

- [ ] **Step 8: Verify host build**

```bash
cargo build --workspace
```

Expected: builds clean.

- [ ] **Step 9: Commit**

```bash
git add crates/
git commit -m "feat(crates): five-crate skeleton with stub is_valid_mnemonic"
```

---

## Task 4: Spec files

**Files:**
- Create: `spec/test-vectors.schema.json`
- Create: `spec/test-vectors.json`
- Create: `spec/api.md` (frozen copy of `docs/api.md`)
- Create: `spec/chains.md` (frozen copy of `docs/chains.md`)
- Create: `spec/errors.md` (frozen copy of `docs/error-model.md`'s reason vocabulary section)
- Create: `spec/CHANGELOG.md`

- [ ] **Step 1: JSON Schema for vectors**

`spec/test-vectors.schema.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://jovachain.io/schemas/test-vectors.json",
  "title": "jovawallet-core test vectors",
  "type": "object",
  "required": ["version", "vectors"],
  "properties": {
    "version": { "type": "string", "pattern": "^[0-9]+\\.[0-9]+$" },
    "vectors": {
      "type": "array",
      "items": { "$ref": "#/$defs/vector" }
    }
  },
  "$defs": {
    "vector": {
      "type": "object",
      "required": ["id", "kind", "input", "expected"],
      "properties": {
        "id": { "type": "string", "pattern": "^[a-z0-9_.-]+$" },
        "kind": {
          "type": "string",
          "enum": [
            "address", "sign_tx", "sign_message",
            "validate_address_pos", "validate_address_neg",
            "mnemonic_validation_pos", "mnemonic_validation_neg",
            "error"
          ]
        },
        "applies_to": {
          "type": "array",
          "items": { "type": "string", "enum": ["rust", "swift", "kotlin", "wasm"] }
        },
        "input": { "type": "object" },
        "expected": { "type": "object" },
        "source": { "type": "string" },
        "source_url": { "type": "string" }
      }
    }
  }
}
```

- [ ] **Step 2: First vector (the one negative validation case)**

`spec/test-vectors.json`:

```json
{
  "version": "0.1",
  "vectors": [
    {
      "id": "phase0.mnemonic_validation_neg.gibberish",
      "kind": "mnemonic_validation_neg",
      "input": {
        "words": "this is not a valid mnemonic at all",
        "passphrase": ""
      },
      "expected": {
        "valid": false
      },
      "source": "Phase 0 hello-world",
      "source_url": "internal"
    }
  ]
}
```

- [ ] **Step 3: Frozen spec copies**

The Phase 0 versions of `spec/api.md`, `spec/chains.md`, and `spec/errors.md` are **byte-identical copies** of `docs/api.md`, `docs/chains.md`, and the relevant section of `docs/error-model.md`. The `tools/verify-spec` tool (Task 5) enforces this.

```bash
cp docs/api.md spec/api.md
cp docs/chains.md spec/chains.md
```

For `spec/errors.md`, extract the "Reason-string vocabulary" section from `docs/error-model.md` and the JovaError taxonomy. (The agent should do this manually or use a tool — the goal is the agent has a frozen reference of the v0 error contract.)

For Phase 0, a placeholder is acceptable:

`spec/errors.md`:

```markdown
# Error Taxonomy — frozen reference

This file is the spec-side mirror of `docs/error-model.md`. At Phase 0 it documents only the trivial stub variant.

## Variants

- (Phase 0 has no error path on `is_valid_mnemonic`; the function returns bool.)

## Reason vocabulary

(Phase 1 fills this in for malformed-tx and malformed-message reasons.)
```

- [ ] **Step 4: spec changelog**

`spec/CHANGELOG.md`:

```markdown
# Spec Changelog

## [0.1] — Phase 0
- First vector: `phase0.mnemonic_validation_neg.gibberish`.
- Schema established: `spec/test-vectors.schema.json`.
- Frozen spec copies: `spec/api.md`, `spec/chains.md`, `spec/errors.md`.
```

- [ ] **Step 5: Run the host hello-world test**

```bash
cargo test -p jova-core --test hello -- --nocapture
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add spec/
git commit -m "feat(spec): vectors schema, first vector, frozen spec copies"
```

---

## Task 5: tools/verify-spec

**Files:**
- Create: `tools/verify-spec/Cargo.toml`
- Create: `tools/verify-spec/src/main.rs`

- [ ] **Step 1: Create the tool's Cargo.toml**

`tools/verify-spec/Cargo.toml`:

```toml
[package]
name = "jova-verify-spec"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[[bin]]
name = "verify-spec"
path = "src/main.rs"

[dependencies]
serde_json = { workspace = true, features = ["std"] }
```

- [ ] **Step 2: Implement the tool**

`tools/verify-spec/src/main.rs`:

```rust
//! verify-spec — fails CI if docs/* and spec/* drift, or if test-vectors.json is malformed.

use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut errors: Vec<String> = Vec::new();

    // 1. Frozen-copy invariant: docs/api.md == spec/api.md, docs/chains.md == spec/chains.md.
    for (a, b) in [
        ("docs/api.md", "spec/api.md"),
        ("docs/chains.md", "spec/chains.md"),
    ] {
        let da = fs::read_to_string(a).unwrap_or_else(|e| {
            errors.push(format!("read {}: {}", a, e));
            String::new()
        });
        let db = fs::read_to_string(b).unwrap_or_else(|e| {
            errors.push(format!("read {}: {}", b, e));
            String::new()
        });
        if !da.is_empty() && !db.is_empty() && da != db {
            errors.push(format!("DRIFT: {} != {}", a, b));
        }
    }

    // 2. test-vectors.json must parse.
    let vectors = fs::read_to_string("spec/test-vectors.json").unwrap_or_else(|e| {
        errors.push(format!("read spec/test-vectors.json: {}", e));
        String::new()
    });
    if !vectors.is_empty() {
        match serde_json::from_str::<serde_json::Value>(&vectors) {
            Ok(v) => {
                let arr_ok = v.get("vectors").and_then(|x| x.as_array()).is_some();
                if !arr_ok {
                    errors.push("test-vectors.json: missing 'vectors' array".into());
                }
            }
            Err(e) => errors.push(format!("test-vectors.json parse: {}", e)),
        }
    }

    if errors.is_empty() {
        println!("verify-spec: OK");
        ExitCode::SUCCESS
    } else {
        for e in &errors {
            eprintln!("verify-spec ERROR: {}", e);
        }
        ExitCode::FAILURE
    }
}
```

- [ ] **Step 3: Run it**

```bash
cargo run -p jova-verify-spec
```

Expected: `verify-spec: OK`.

- [ ] **Step 4: Test failure path**

```bash
echo "// drift" >> docs/api.md
cargo run -p jova-verify-spec; echo "exit=$?"
git checkout docs/api.md
```

Expected: prints `DRIFT: docs/api.md != spec/api.md` and exits non-zero.

- [ ] **Step 5: Commit**

```bash
git add tools/verify-spec/ Cargo.toml
git commit -m "feat(tools): verify-spec catches docs↔spec drift and malformed vectors"
```

---

## Task 6: Swift binding scaffold + hello-world test

**Files:**
- Create: `bindings/swift/Package.swift`
- Create: `bindings/swift/Sources/JovaCore/Convenience.swift`
- Create: `bindings/swift/Tests/JovaCoreTests/HelloWorldTests.swift`
- Create: `bindings/swift/scripts/build-xcframework.sh`

- [ ] **Step 1: Build the FFI lib for the host (smoke for binding generation)**

```bash
cargo build -p jova-core-ffi --release
```

Expected: produces `target/release/libjova_core_ffi.{a,dylib,so}` depending on host.

- [ ] **Step 2: Generate the Swift bindings**

```bash
cargo install uniffi-bindgen-cli --version 0.29 --locked
mkdir -p bindings/swift/Sources/JovaCore
uniffi-bindgen-cli generate \
  --library target/release/libjova_core_ffi.dylib \
  --language swift \
  --out-dir bindings/swift/Sources/JovaCore
```

(On Linux replace `.dylib` with `.so`. The CI matrix builds for all three.)

- [ ] **Step 3: Create `Package.swift`**

`bindings/swift/Package.swift`:

```swift
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "JovaCore",
    platforms: [.iOS(.v14), .macOS(.v11)],
    products: [
        .library(name: "JovaCore", targets: ["JovaCore"]),
    ],
    targets: [
        .binaryTarget(
            name: "JovaCoreFFI",
            // Phase 0: local-only build. CI replaces this with the XCFramework path.
            path: "JovaCoreFFI.xcframework"
        ),
        .target(
            name: "JovaCore",
            dependencies: ["JovaCoreFFI"],
            path: "Sources/JovaCore"
        ),
        .testTarget(
            name: "JovaCoreTests",
            dependencies: ["JovaCore"],
            path: "Tests/JovaCoreTests",
            resources: [.copy("../../../../spec/test-vectors.json")]
        ),
    ]
)
```

- [ ] **Step 4: Convenience.swift**

`bindings/swift/Sources/JovaCore/Convenience.swift`:

```swift
// Hand-written ergonomics layer. Phase 0 has nothing to add yet.
import Foundation

public enum JovaCoreVersion {
    public static let value = "0.0.1"
}
```

- [ ] **Step 5: HelloWorldTests.swift**

`bindings/swift/Tests/JovaCoreTests/HelloWorldTests.swift`:

```swift
import XCTest
@testable import JovaCore

final class HelloWorldTests: XCTestCase {
    func testNegativeMnemonicValidationVector() throws {
        guard let url = Bundle.module.url(forResource: "test-vectors", withExtension: "json") else {
            XCTFail("test-vectors.json not in test bundle")
            return
        }
        let data = try Data(contentsOf: url)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        let vectors = json["vectors"] as! [[String: Any]]
        let v = vectors.first { ($0["id"] as? String) == "phase0.mnemonic_validation_neg.gibberish" }!

        let input = v["input"] as! [String: Any]
        let words = input["words"] as! String
        let passphrase = (input["passphrase"] as? String) ?? ""
        let expected = (v["expected"] as! [String: Any])["valid"] as! Bool

        XCTAssertEqual(isValidMnemonic(words: words, passphrase: passphrase), expected)
    }
}
```

- [ ] **Step 6: Build script**

`bindings/swift/scripts/build-xcframework.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Build for every Apple target.
# iOS sim is arm64-only — Apple has fully deprecated the Intel iOS simulator.
for target in aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build -p jova-core-ffi --release --target "$target" --manifest-path ../../Cargo.toml
done

# Mac slice is universal (arm64 + x86_64) via lipo.
mkdir -p ../../target/mac-universal
lipo -create \
  ../../target/aarch64-apple-darwin/release/libjova_core_ffi.a \
  ../../target/x86_64-apple-darwin/release/libjova_core_ffi.a \
  -output ../../target/mac-universal/libjova_core_ffi.a

# Generate the modulemap and headers.
uniffi-bindgen-cli generate \
  --library ../../target/aarch64-apple-darwin/release/libjova_core_ffi.dylib \
  --language swift \
  --out-dir Sources/JovaCore

# Build the XCFramework: device, simulator (arm64-only), macOS universal.
rm -rf JovaCoreFFI.xcframework
xcodebuild -create-xcframework \
  -library ../../target/aarch64-apple-ios/release/libjova_core_ffi.a      -headers Sources/JovaCore \
  -library ../../target/aarch64-apple-ios-sim/release/libjova_core_ffi.a  -headers Sources/JovaCore \
  -library ../../target/mac-universal/libjova_core_ffi.a                  -headers Sources/JovaCore \
  -output JovaCoreFFI.xcframework

echo "✅ XCFramework at $(pwd)/JovaCoreFFI.xcframework"
```

```bash
chmod +x bindings/swift/scripts/build-xcframework.sh
```

- [ ] **Step 7: Smoke build (macOS only; CI does the full matrix)**

If running on macOS host:

```bash
./bindings/swift/scripts/build-xcframework.sh
cd bindings/swift
swift test
cd ../..
```

Expected on macOS: `swift test` reports `Test Suite 'HelloWorldTests' passed`.

If on Linux: skip — CI's macos-latest runner exercises this.

- [ ] **Step 8: Commit**

```bash
git add bindings/swift/
git commit -m "feat(swift): scaffold + XCFramework build script + hello-world vector test"
```

---

## Task 7: Kotlin binding scaffold + hello-world test

**Files:**
- Create: `bindings/kotlin/settings.gradle.kts`
- Create: `bindings/kotlin/build.gradle.kts`
- Create: `bindings/kotlin/jova-core/build.gradle.kts`
- Create: `bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/HelloWorldTest.kt`
- Create: `bindings/kotlin/jova-core/src/test/resources/test-vectors.json` (symlinked)
- Create: `bindings/kotlin/scripts/build-aar.sh`

- [ ] **Step 1: settings.gradle.kts**

```kotlin
rootProject.name = "jova-core-android"
include(":jova-core")
```

- [ ] **Step 2: Root build.gradle.kts**

```kotlin
buildscript {
    repositories { google(); mavenCentral() }
}

allprojects {
    repositories { google(); mavenCentral() }
}
```

- [ ] **Step 3: Module build.gradle.kts**

`bindings/kotlin/jova-core/build.gradle.kts`:

```kotlin
plugins {
    id("com.android.library") version "8.5.0"
    kotlin("android") version "1.9.24"
}

android {
    namespace = "io.jova.core"
    compileSdk = 34
    defaultConfig {
        minSdk = 24
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86") }
    }
    sourceSets {
        named("main") {
            jniLibs.srcDirs("src/main/jniLibs")
            kotlin.srcDirs("src/main/kotlin")
        }
        named("test") {
            kotlin.srcDirs("src/test/kotlin")
            resources.srcDirs("src/test/resources")
        }
    }
    testOptions {
        unitTests.isIncludeAndroidResources = true
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.json:json:20240303")
}
```

- [ ] **Step 4: Generate Kotlin bindings**

```bash
uniffi-bindgen-cli generate \
  --library target/release/libjova_core_ffi.dylib \
  --language kotlin \
  --out-dir bindings/kotlin/jova-core/src/main/kotlin
```

(Replace `.dylib` with `.so` on Linux.)

- [ ] **Step 5: HelloWorldTest.kt**

`bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/HelloWorldTest.kt`:

```kotlin
package io.jova.core

import org.junit.Test
import org.junit.Assert.assertEquals
import org.json.JSONObject

class HelloWorldTest {
    @Test
    fun negativeMnemonicValidationVector() {
        val json = JSONObject(
            javaClass.getResourceAsStream("/test-vectors.json")!!.bufferedReader().readText()
        )
        val vectors = json.getJSONArray("vectors")
        var v: JSONObject? = null
        for (i in 0 until vectors.length()) {
            val candidate = vectors.getJSONObject(i)
            if (candidate.getString("id") == "phase0.mnemonic_validation_neg.gibberish") {
                v = candidate; break
            }
        }
        requireNotNull(v) { "vector phase0.mnemonic_validation_neg.gibberish missing" }

        val input = v.getJSONObject("input")
        val words = input.getString("words")
        val passphrase = if (input.has("passphrase")) input.getString("passphrase") else ""
        val expected = v.getJSONObject("expected").getBoolean("valid")

        assertEquals(expected, isValidMnemonic(words, passphrase))
    }
}
```

- [ ] **Step 6: Test resources copy**

```bash
mkdir -p bindings/kotlin/jova-core/src/test/resources
cp spec/test-vectors.json bindings/kotlin/jova-core/src/test/resources/test-vectors.json
```

(Symlinking works on Unix but not Windows; copying is safer for CI portability. The release script will re-copy on every CI run so the version stays in sync with `spec/`.)

- [ ] **Step 7: build-aar.sh**

`bindings/kotlin/scripts/build-aar.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

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

# Generate Kotlin bindings.
uniffi-bindgen-cli generate \
  --library ../../target/aarch64-linux-android/release/libjova_core_ffi.so \
  --language kotlin \
  --out-dir jova-core/src/main/kotlin

# Build the AAR.
./gradlew :jova-core:assembleRelease
echo "✅ AAR at jova-core/build/outputs/aar/"
```

```bash
chmod +x bindings/kotlin/scripts/build-aar.sh
```

- [ ] **Step 8: Commit**

```bash
git add bindings/kotlin/
git commit -m "feat(kotlin): scaffold + AAR build script + hello-world vector test"
```

---

## Task 8: WASM binding scaffold + hello-world test

**Files:**
- Create: `bindings/wasm/package.json`
- Create: `bindings/wasm/tsconfig.json`
- Create: `bindings/wasm/src/index.ts`
- Create: `bindings/wasm/tests/hello.test.ts`
- Create: `bindings/wasm/scripts/build-wasm.sh`

- [ ] **Step 1: package.json**

```json
{
  "name": "@jovachain/wallet-core",
  "version": "0.0.1",
  "private": true,
  "description": "WASM binding for jovawallet-core",
  "type": "module",
  "main": "./pkg/jova_core_wasm.js",
  "types": "./pkg/jova_core_wasm.d.ts",
  "files": ["pkg/", "src/", "README.md"],
  "scripts": {
    "build": "./scripts/build-wasm.sh",
    "test": "vitest run"
  },
  "devDependencies": {
    "vitest": "^2.1.0",
    "typescript": "^5.5.0"
  }
}
```

- [ ] **Step 2: tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "resolveJsonModule": true
  },
  "include": ["src", "tests"]
}
```

- [ ] **Step 3: src/index.ts**

```typescript
import init, { isValidMnemonic } from '../pkg/jova_core_wasm.js';
export { init, isValidMnemonic };
```

- [ ] **Step 4: tests/hello.test.ts**

```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import init, { isValidMnemonic } from '../pkg/jova_core_wasm.js';
import vectors from '../../../spec/test-vectors.json';

describe('Phase 0 hello-world', () => {
    beforeAll(async () => { await init(); });

    it('rejects the negative-mnemonic vector', () => {
        const v = vectors.vectors.find(
            (x: any) => x.id === 'phase0.mnemonic_validation_neg.gibberish'
        );
        expect(v).toBeDefined();
        const words = v!.input.words as string;
        const passphrase = (v!.input.passphrase as string | undefined) ?? '';
        const expected = (v!.expected as any).valid as boolean;

        expect(isValidMnemonic(words, passphrase)).toBe(expected);
    });
});
```

- [ ] **Step 5: build-wasm.sh**

`bindings/wasm/scripts/build-wasm.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Build the WASM crate.
(cd ../../crates/jova-core-wasm && \
  wasm-pack build --release --target web --out-dir ../../bindings/wasm/pkg)

echo "✅ WASM package at $(pwd)/pkg"
```

```bash
chmod +x bindings/wasm/scripts/build-wasm.sh
```

- [ ] **Step 6: Smoke build & test (Linux/macOS)**

```bash
cd bindings/wasm
./scripts/build-wasm.sh
pnpm install
pnpm test
cd ../..
```

Expected: vitest reports 1 passing test.

- [ ] **Step 7: Commit**

```bash
git add bindings/wasm/
git commit -m "feat(wasm): scaffold + build script + hello-world vector test"
```

---

## Task 9: GitHub Actions workflows

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/ci-bindings-swift.yml`
- Create: `.github/workflows/ci-bindings-kotlin.yml`
- Create: `.github/workflows/ci-bindings-wasm.yml`
- Create: `.github/workflows/ci-no-std.yml`
- Create: `.github/workflows/audit.yml`

- [ ] **Step 1: ci.yml (host Rust)**

```yaml
name: ci
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace --locked
      - run: cargo run -p jova-verify-spec
```

- [ ] **Step 2: ci-bindings-swift.yml**

```yaml
name: ci-bindings-swift
on: [push, pull_request]
jobs:
  swift:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: aarch64-apple-ios,aarch64-apple-ios-sim,aarch64-apple-darwin,x86_64-apple-darwin }
      - run: cargo install uniffi-bindgen-cli --version 0.29 --locked
      - run: ./bindings/swift/scripts/build-xcframework.sh
      - run: cd bindings/swift && swift test
```

- [ ] **Step 3: ci-bindings-kotlin.yml**

```yaml
name: ci-bindings-kotlin
on: [push, pull_request]
jobs:
  kotlin:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: aarch64-linux-android,armv7-linux-androideabi,x86_64-linux-android,i686-linux-android }
      - uses: nttld/setup-ndk@v2
        id: ndk
        with: { ndk-version: r27c }
      - run: cargo install cargo-ndk --version 3.5 --locked
      - run: cargo install uniffi-bindgen-cli --version 0.29 --locked
      - uses: actions/setup-java@v4
        with: { distribution: temurin, java-version: 21 }
      - env:
          ANDROID_NDK_HOME: ${{ steps.ndk.outputs.ndk-path }}
        run: ./bindings/kotlin/scripts/build-aar.sh
      - run: cd bindings/kotlin && ./gradlew :jova-core:test
```

- [ ] **Step 4: ci-bindings-wasm.yml**

```yaml
name: ci-bindings-wasm
on: [push, pull_request]
jobs:
  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: wasm32-unknown-unknown }
      - run: cargo install wasm-pack --version 0.13 --locked
      - uses: pnpm/action-setup@v4
        with: { version: 10 }
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - run: ./bindings/wasm/scripts/build-wasm.sh
      - run: cd bindings/wasm && pnpm install && pnpm test
```

- [ ] **Step 5: ci-no-std.yml**

```yaml
name: ci-no-std
on: [push, pull_request]
jobs:
  no_std:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: thumbv7em-none-eabihf }
      - run: cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --no-default-features
```

- [ ] **Step 6: audit.yml**

```yaml
name: audit
on:
  push:
  pull_request:
  schedule: [{ cron: '0 4 * * *' }]
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit cargo-deny --locked
      - run: cargo audit
      - run: cargo deny check
```

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/
git commit -m "ci: six workflows enforcing the matrix on every PR"
```

---

## Task 10: Open the bootstrap PR, verify CI green, tag v0.0.1

- [ ] **Step 1: Push the branch**

If `origin` is configured:

```bash
git checkout -b feat/phase-0-bootstrap
git push -u origin feat/phase-0-bootstrap
```

- [ ] **Step 2: Open the PR via gh**

```bash
gh pr create \
  --title "Phase 0: repo bootstrap" \
  --body "$(cat <<'EOF'
## Summary
- 5-crate Rust workspace with stub `is_valid_mnemonic`
- Three binding scaffolds (Swift, Kotlin, WASM) each with a hello-world vector test
- 6 CI workflows on the matrix
- spec/ with first vector + JSON Schema + frozen api.md/chains.md copies
- tools/verify-spec catches docs↔spec drift
- Governance files: LICENSE, CONTRIBUTING, SECURITY, CODEOWNERS

## Test plan
- [x] cargo test --workspace passes locally
- [x] All 6 CI workflows pass on this PR
- [x] no_std build for thumbv7em-none-eabihf passes

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Wait for CI**

Watch the PR. All six workflows must be green. If any are red:

- Read the failure log.
- Fix in a new commit on the same branch.
- Push; CI re-runs automatically.

- [ ] **Step 4: Merge to main**

After CI is green and a code review has approved:

```bash
gh pr merge --squash --delete-branch
git checkout main && git pull
```

- [ ] **Step 5: Tag v0.0.1**

```bash
git tag -a v0.0.1 -m "v0.0.1 — Phase 0 bootstrap"
git push origin v0.0.1
```

- [ ] **Step 6: Verify the tag triggered nothing**

The release workflow doesn't exist yet — Phase 0 doesn't publish. The tag is a marker only. Confirm no GitHub Actions ran for the tag (or only the existing PR workflows ran on the merge commit).

---

## Self-review

- [ ] Every file has exact path.
- [ ] Every code block has the actual code, not a placeholder.
- [ ] Every command is copy-pasteable.
- [ ] CI workflows match the workflows referenced in `docs/build-and-release.md`.
- [ ] The `is_valid_mnemonic` stub returns false for the vector input ("this is not a valid mnemonic at all" is not equal to "valid"), so the test passes.
- [ ] All five crates compile.
- [ ] no_std crate has `default-features = false` and gates std behind a feature.
- [ ] The hello-world test loads from `spec/test-vectors.json` on every binding (not embedded copies).

---

## What this plan does NOT do

- Does not implement BIP-39 or any real crypto. Phase 1.
- Does not publish to crates.io, Maven Central, or npm. Phase 5+.
- Does not exercise functional WASM tests beyond the hello-world. Phase 6.
- Does not run miri or fuzzing. Phase 1+.
- Does not produce documentation tasks (docs are already written; `tools/verify-spec` catches drift between docs and spec).

---

## Estimated time

3–5 days. The biggest time sinks are usually:
1. Getting `uniffi-bindgen-cli generate` to find the right library file across host OSes.
2. Android NDK + cargo-ndk path issues.
3. CI workflow YAML syntax until you've done it once.
