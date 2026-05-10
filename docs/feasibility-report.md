# Feasibility Report — Phase -1

**Date:** 2026-05-11
**Branch:** `spike/feasibility`
**Commit SHA:** `9b11688`

---

## Targets exercised

| Target | Spike crate | Result | Notes |
|---|---|---|---|
| `aarch64-apple-ios` | jova-spike | Pass | All 4 chains (evm+btc+sol+xrp) with `--features ffi` |
| `aarch64-apple-ios-sim` | jova-spike | Pass | All 4 chains with `--features ffi` |
| `aarch64-apple-darwin` | jova-spike | Pass | All 4 chains; also used as uniffi metadata source for Kotlin gen |
| `x86_64-apple-darwin` | jova-spike | Pass | All 4 chains with `--features ffi` |
| `aarch64-linux-android` (arm64-v8a) | jova-spike | Pass | 351K `.so`; all 4 chains; cargo-ndk |
| `armv7-linux-androideabi` (armeabi-v7a) | jova-spike | Pass | 252K `.so`; historically the trickiest ABI; built clean |
| `x86_64-linux-android` (x86_64) | jova-spike | Pass | 380K `.so` |
| `i686-linux-android` (x86) | jova-spike | Pass | 381K `.so` |
| `wasm32-unknown-unknown` baseline | jova-spike-wasm | Pass | alloy + primitives, bitcoin optional off; 13K `.wasm`; Node smoke passes |
| `wasm32-unknown-unknown` + chain-btc | jova-spike-wasm | Fail | `secp256k1-sys` requires a C compiler with WASM backend; Apple clang has none |
| `wasm32-unknown-unknown` + chain-sol | jova-spike-wasm | Pass | Anza split crates compile after three-way `getrandom` feature force-enable |
| `wasm32-unknown-unknown` + chain-xrp | jova-spike-wasm | Fail | `xrpl-rust` 1.1 transitively requires `secp256k1-sys`; same root cause as chain-btc |
| `thumbv7em-none-eabihf` | jova-spike-nostd | Pass | All 5 primitive crates; `arm-none-eabi-gcc` 10.3 (with newlib) required |

---

## Per-crate findings

### `bdk_wallet`

- **Native (iOS, Android, macOS):** Pass
- **WASM:** Fail (inherits `secp256k1-sys` from `bitcoin` v0.32)
- **no_std:** Not applicable (`bdk_wallet` is std-only; see note)
- **Notes:**
  - `default-features = false` alone is insufficient. The `miniscript` transitive dependency requires `std`, so the correct workspace declaration is `default-features = false, features = ["std"]`.
  - `bdk_wallet` is std-only. It cannot be used in the no_std primitives crate. This is a Phase 7 (hardware-wallet firmware) concern — hardware signing for BTC will need a different path (e.g., raw `bitcoin` crate with `no_std` feature or an alternative). No action required for Phase 0.
  - WASM failure root cause: `bitcoin` v0.32 depends on `secp256k1-sys` (C-backed). Apple clang does not include a WASM backend, so `cc-rs` cannot compile the C sources for the `wasm32-unknown-unknown` target.

### `alloy`

- **Native (iOS, Android, macOS):** Pass
- **WASM:** Pass
- **Notes:**
  - Uses pure-Rust `k256` for secp256k1 operations (no C FFI). This is the reason alloy works on WASM while `bitcoin` and `xrpl-rust` do not.
  - Features used: `consensus`, `signer-local`, `sol-types`, `dyn-abi` — sufficient for EVM transaction signing and ABI encoding.
  - No issues encountered on any target.

### Anza Solana split crates

Tested as a bundle: `solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`.

- **Native (iOS, Android, macOS):** Pass — all five crates
- **WASM:** Pass — all five crates, with a workaround required
- **Notes:**
  - The dep tree contains three concurrent `getrandom` versions: 0.2 (from `alloy`/`k256`/`elliptic-curve`), 0.3 (from Solana's `rand` 0.9), and 0.4 (from `wasm-pack` boilerplate). Each version has a different WASM feature name (`js` vs `wasm_js`). The WASM crate's `Cargo.toml` carries explicit direct-dep aliases to force-enable the correct feature for each version.
  - This three-way `getrandom` explosion is a known transient issue in the crate ecosystem. The spike workaround is functional but fragile — if any upstream crate bumps `getrandom`, the alias may need updating. A long-term solution is needed before Phase 6 (WASM release).
  - No tokio or `std::os::unix` is pulled in; the Anza split crates are leaner than the monolithic `solana-sdk` and do not fight the WASM target.
  - Crates confirmed compatible with `default-features = false` on all targets.

### `xrpl-rust` (crate name on crates.io: `xrpl-rust`, declared in `Cargo.toml` as `xrpl-rust`)

- **Native (iOS, Android, macOS):** Pass
- **WASM:** Fail
- **Notes:**
  - **Critical correction from spike Task 2/4:** the original spike Cargo.toml referenced `xrpl = "0.1.2"`, which is an entirely different crate — a minimal async WebSocket client (`XrplClient`, `request`, `socket`, `subscriptions`, `types` modules) with no signing functionality. This is NOT the library named in ADR-1 or `docs/chains.md`. The spike corrected this to `xrpl-rust = "1.1"` in Task 4, matching the ADR choice. All Phase 0 plan text that references `xrpl` as a crate name should be updated to `xrpl-rust`.
  - Features used: `core`, `wallet`, `models` with `default-features = false`.
  - WASM failure root cause: `xrpl-rust` 1.1 transitively depends on `secp256k1` (C-backed via `secp256k1-sys`). Same root cause as `bdk_wallet`/`bitcoin` WASM failure — no WASM-capable C compiler in the default macOS toolchain.
  - The lib.rs reference uses `xrpl::wallet::Wallet` — the crate exposes itself as `xrpl` despite being named `xrpl-rust` on crates.io.
  - `xrpl-rust` does not expose a `secp256k1-sys` bypass feature (no `pure-rust` or `k256` feature gate as of 1.1).

### `secp256k1` / `ed25519-dalek` / `bip39` / `slip-10` / `zeroize`

- **Native (iOS, Android, macOS):** Pass
- **WASM:** Not tested as a standalone unit (used via the WASM baseline/chain-sol path; alloy and Anza crates bring their own copies of some of these)
- **no_std (`thumbv7em-none-eabihf`):** Pass — all five crates
- **Notes:**
  - `bip39 2.2.x`: the plan specified `features = ["english"]` but no such feature exists in this version. The English wordlist is always-on. Corrected to `features = ["alloc"]`.
  - `secp256k1 0.31`: workspace dep must NOT include `global-context` in the shared feature list. `global-context` requires `std`, and Cargo's feature unification would propagate it into the no_std crate. Anything needing `global-context` must enable it at the individual call site. Workspace now declares `features = ["alloc", "lowmemory"]`.
  - `secp256k1-sys` compiles C sources via `cc-rs`. For `thumbv7em-none-eabihf`, this requires `arm-none-eabi-gcc` with newlib. The official Arm GNU Embedded Toolchain (ArmMbed homebrew formulae, 10.3-2021.10) works. Homebrew core's `arm-none-eabi-gcc` 16.x does NOT ship newlib and fails. CI uses `apt-get install -y gcc-arm-none-eabi`, which provides the correct toolchain on Linux. This is a firm Phase 7 (hardware) build prerequisite.
  - `slip-10 0.4` and `zeroize 1.8` compile clean with `default-features = false` on all targets with no adaptation needed.

---

## Decisions

### chain-btc on WASM

Root cause: `bitcoin` v0.32 depends on `secp256k1-sys`, which wraps the libsecp256k1 C library. `cc-rs` cannot compile C for `wasm32-unknown-unknown` using Apple clang — Apple's clang distribution does not include the WASM backend (`clang --target wasm32-unknown-unknown` fails with no such target). `bdk_wallet` inherits this failure.

Three options for Phase 6 (WASM release):

- **(a) Pure-Rust secp256k1 swap:** use `k256` (from `RustCrypto`) for Bitcoin signing on WASM. `k256` is already in the tree via `alloy` and is WASM-compatible. This requires a custom BTC transaction signer that bypasses `bitcoin`/`bdk_wallet` for the WASM build path. Significant implementation work but keeps BTC on WASM.
- **(b) Emscripten in CI:** install Emscripten (`emcc`) and configure `cc-rs` to use it for the WASM target. This would let `secp256k1-sys` compile for WASM without code changes. Adds CI complexity (Emscripten is large); local developer builds also need Emscripten.
- **(c) Ship WASM v1.1 with EVM + SOL only; add BTC and XRP in a later WASM release:** feature-flag chain-btc and chain-xrp out of the WASM build at v1.1 launch. BTC and XRP users on web fall back to backend signing at v1.1 and gain client-side WASM support in a follow-up release. Simplest for Phase 6; defers the hard problem.

**Resolution (2026-05-11, user decision):** option **(c)**. BTC and XRP are required for the mobile wallet and remain full-featured on iOS and Android native targets. Browser WASM BTC/XRP signing is **deferred** beyond v1.1 — Phase 6 ships WASM with EVM + SOL only. Phase 0 should structure `jova-core-wasm` so that `chain-btc` and `chain-xrp` are feature-flagged off for the WASM build path, leaving the door open for a later WASM release once a pure-Rust secp256k1 swap or Emscripten path is chosen.

### chain-xrp on WASM

Root cause: `xrpl-rust` 1.1 transitively requires `secp256k1-sys`. Same C-compiler constraint as chain-btc.

Options for Phase 6:

- **(a) Pure-Rust secp256k1 swap:** implement XRP signing using `k256` directly (XRP uses secp256k1 and ed25519). Would require vendoring or contributing a WASM-capable XRP signing path outside of `xrpl-rust`. Feasible but non-trivial.
- **(b) Emscripten in CI:** same as chain-btc option (b) above.
- **(c) Ship WASM v1.1 without XRP:** same as chain-btc option (c) above.

**Resolution (2026-05-11, user decision):** option **(c)**, matching the chain-btc decision above. XRP signing remains full-featured on mobile (iOS, Android, native macOS). Browser WASM XRP support is deferred beyond v1.1.

### chain-sol on WASM

No decision needed. Passes with the `getrandom` workaround in place. The long-term `getrandom` strategy is an open question (see below) but is not a blocker.

### Android uniffi Kotlin generation

Not a failure, but a process decision confirmed by the spike: `uniffi-bindgen` cannot `dlopen` a cross-compiled ELF `.so` on macOS (different ABI). The workaround — generate Kotlin from the native `aarch64-apple-darwin` dylib — is correct because uniffi metadata is ABI-independent. Phase 0 build scripts should document and encode this pattern.

---

## Recommended Phase 0 dependency configuration

The following `[workspace.dependencies]` block reflects all corrections made during the spike and is confirmed to work for native (iOS, Android, macOS) and for the WASM baseline + chain-sol path. Copy this into the Phase 0 `Cargo.toml` as the starting point.

```toml
[workspace.dependencies]
# EVM
alloy             = { version = "2.0",  default-features = false, features = ["consensus", "signer-local", "sol-types", "dyn-abi"] }

# Bitcoin
bdk_wallet        = { version = "3.0",  default-features = false, features = ["std"] }
bitcoin           = { version = "0.32", default-features = false, features = ["secp-recovery"] }

# Solana (Anza split crates — leaner than monolithic solana-sdk)
solana-keypair    = { version = "3.1", default-features = false }
solana-pubkey     = { version = "4.2", default-features = false }
solana-signature  = { version = "3.4", default-features = false }
solana-transaction = { version = "4.1", default-features = false }
solana-message    = { version = "4.1", default-features = false }

# XRP — NOTE: crate name on crates.io is `xrpl-rust`; exposes as `xrpl` in code
xrpl-rust         = { version = "1.1", default-features = false, features = ["core", "wallet", "models"] }

# Primitives (no_std-compatible)
# NOTE: do NOT add global-context here — it requires std and poisons no_std via feature unification
secp256k1         = { version = "0.31", default-features = false, features = ["alloc", "lowmemory"] }
ed25519-dalek     = { version = "2.2",  default-features = false, features = ["alloc"] }
# NOTE: bip39 2.2.x has no "english" feature — English wordlist is always-on; use "alloc"
bip39             = { version = "2.2",  default-features = false, features = ["alloc"] }
slip-10           = { version = "0.4",  default-features = false }
zeroize           = { version = "1.8",  default-features = false, features = ["alloc"] }

# Bindings
uniffi            = { version = "0.31", features = ["build", "cli"] }
wasm-bindgen      = { version = "0.2" }
serde-wasm-bindgen = { version = "0.6" }

# Utilities
serde             = { version = "1", default-features = false, features = ["derive", "alloc"] }
serde_json        = { version = "1", default-features = false, features = ["alloc"] }
thiserror         = "2"
hex               = { version = "0.4", default-features = false, features = ["alloc"] }
```

**Corrections vs. original plan text:**
1. `xrpl-rust = "1.1"` replaces `xrpl = "0.1.2"` (wrong crate).
2. `bip39` features: `["alloc"]` replaces `["english"]` (feature does not exist).
3. `bdk_wallet` features: `["std"]` added (`default-features = false` alone fails due to `miniscript`).
4. `secp256k1` features: `global-context` removed from workspace declaration.

---

## Open questions

The following require a decision from the user before or during Phase 0/6:

1. **WASM strategy for BTC and XRP signing.** The spike confirmed that `secp256k1-sys` cannot compile for WASM with the default macOS toolchain. Three options were presented above (pure-Rust k256 swap, Emscripten in CI, or ship EVM+SOL-only WASM at v1.1). **The user must choose before Phase 6 begins.** Phase 0 can proceed without a decision, but the WASM crate architecture in Phase 0 should avoid prematurely committing to one path.

2. **Plan/spec text audit for `xrpl` → `xrpl-rust`.** Multiple plan files, `docs/chains.md`, and `docs/api.md` reference `xrpl` as if it is the crate name. All occurrences that refer to the dependency declaration in `Cargo.toml` should be updated to `xrpl-rust`. (Code references using `use xrpl::...` remain unchanged — the crate exposes itself as `xrpl` at the Rust module level.) Recommend a search-and-update pass at Phase 0 start.

3. **`arm-none-eabi-gcc` with newlib for Phase 7 (hardware) builds.** `secp256k1-sys` requires a C compiler with newlib for `thumbv7em-none-eabihf`. Homebrew core's `arm-none-eabi-gcc` 16.x does NOT work (no newlib). The confirmed working tool is `armmbed/formulae/arm-none-eabi-gcc` 10.3-2021.10 on macOS and `gcc-arm-none-eabi` via apt on Linux. This must be documented in `docs/env-setup.md` and added to Phase 7 prerequisites.

4. **`getrandom` version explosion — long-term solution.** The WASM dep tree currently holds three concurrent `getrandom` versions (0.2, 0.3, 0.4) requiring explicit per-version feature aliases in the WASM crate's `Cargo.toml`. Options: wait for upstream crates to converge on a single `getrandom` version; apply `[patch.crates-io]` to force a single version (may break API); or fork and maintain patched copies. Recommend monitoring upstream before Phase 6 and reassessing at that point.

5. **`wasm-opt` / binaryen for production WASM builds.** `wasm-pack` 0.14's bundled `wasm-opt` cannot optimize bulk-memory operations emitted by rustc 1.95 (optimization crashes). The spike workaround is `wasm-opt = false` in `[package.metadata.wasm-pack.profile.release]`. Production WASM builds for Phase 6 will need a current binaryen installed separately in CI (not bundled with wasm-pack). Plan for this in the Phase 6 CI setup.

6. **uniffi-bindgen Swift filename convention.** `uniffi-bindgen` 0.30+ generates `jova_spike.swift` (lowercase, underscores). Plan text written before the 0.30 release may reference `JovaSpike.swift` (PascalCase). Update generated-file references in Phase 0 plan files and iOS integration docs to use the lowercase underscore form.

7. **CI cold-run performance.** The committed CI workflow (`.github/workflows/spike-feasibility.yml`) has no caching for `~/.cargo/bin`, `~/.cargo/registry`, or build artifacts. Cold runs will be slow: `cargo install uniffi --features cli` runs on every iOS and Android job. Phase 0 CI work should add `actions/cache` for cargo registry and target artifacts, and consider pre-installing tools in a base Docker image for Android jobs.

---

## Go / no-go for Phase 0

| Criterion | Result |
|---|---|
| All native targets compile every chain | Pass — 4 Apple targets + 4 Android ABIs, all chains, clean |
| WASM compiles at least baseline (no chains) | Pass — 13K `.wasm`, Node smoke passes |
| Per-chain WASM situation is documented (pass / replace / defer) | Pass — chain-sol passes; chain-btc and chain-xrp fail with root cause documented and three options presented |
| no_std primitives build clean | Pass — all 5 primitives on `thumbv7em-none-eabihf` |

**All four criteria are met.**

**Recommendation: GO for Phase 0**, with the following conditions:

- The WASM strategy for BTC and XRP (option a/b/c above) is deferred to the user's decision before Phase 6. Phase 0 does not require a choice; the WASM crate in Phase 0 should leave the door open for any of the three options.
- The `xrpl` → `xrpl-rust` crate name correction should be propagated through plan files at the start of Phase 0.
- The corrected workspace dependency block above (not the original plan text) should be used as the Phase 0 `Cargo.toml` starting point.
- The `arm-none-eabi-gcc` newlib requirement should be added to `docs/env-setup.md` before Phase 7 begins.

The spike crates (`jova-spike`, `jova-spike-wasm`, `jova-spike-nostd`) are throwaway. Phase 0 starts from `main`, which has not received any spike commits. The `spike/feasibility` branch remains as a historical record.
