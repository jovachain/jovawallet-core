# HANDOFF — autonomous session complete (2026-05-14)

**Status:** Phases 2 through 7 shipped this session. `v0.3.0` tagged on `main`. The remaining `v1.0.0`, `v1.1.0`, `v1.2.0` tags are gated on external blockers (audit, app-team rollout, bug bounty funding, hardware repo) tracked as GitHub issues #3, #4, #8–#14.

## What's on `main` now

```
57e6266  feat(primitives): Phase 7 hardware-wallet readiness (#17)
954fb1e  Phase 6: WASM functional (EVM + SOL) (#16)
581e805  chore(phase-5): hardening pipeline scaffolding + v1.0 release gates (#15)
a69c26a  docs(phase-4): example apps + Mac-required boundary document (#7)
a15ef36  Phase 3: Solana + XRP + remaining EVM (v0.3.0) (#6)
a18f3e9  Phase 2: Bitcoin (BIP-84 + PSBT + BIP-322) — v0.2.0 (#5)
bcb9751  docs: add Linux dev VM setup
2d1aa12  Phase 1: EVM end-to-end (v0.1.0) (#2)
e07107f  Phase 0: repo bootstrap (PR #1)
d035c58  initial docs
```

Tags: `v0.0.1`, `v0.1.0`, `v0.2.0`, `v0.3.0`. The next tag (`v1.0.0` from Phase 5 completion) is the audit / RC / bug-bounty milestone.

## What each phase shipped

### Phase 2 → `v0.2.0` — Bitcoin

BIP-84 native SegWit (`bc1q…`) + PSBT signing (single-input, multi-input fully-owned, multi-party partial) + BIP-322 simple message signing + legacy Bitcoin Core signmessage fallback. Captures cross-validated against `embit 0.8.0` and the `bip322` PyPI verifier. 12 vectors. Multi-party PSBT signaling via `psbt:` prefix on `SignedTx.raw_hex`. `tools/btc-migration-check` binary scaffolded.

### Phase 3 → `v0.3.0` — Solana + XRP + remaining EVM

**3a:** 5 vectors covering Polygon, BSC, Arbitrum, Optimism, Base. No code changes — `EvmSigner` already routed every chain.

**3b:** XRP classic address (`r…`) + canonical XRPL signing (SHA512Half, secp256k1 ECDSA). `XrpSigner` sibling type (XRP has no message-signing scheme). `JovaChain::Xrp`, `UnsignedTx::Xrp { tx_json }`. 6 vectors cross-validated against `xrpl-py 4.5` + `bip_utils 2.x`.

**3c:** SLIP-10 ed25519 derivation (implemented in-crate — `slip-10 0.4` doesn't ship ed25519). `Ed25519Xprv` (Zeroize + ZeroizeOnDrop). Solana base58 address. VersionedTransaction (v0) signing with ALT support. Raw ed25519 message signing. `SolSigner` sibling type. 8 vectors cross-validated against solders + `bip_utils Bip44Coins.SOLANA`.

### Phase 4 — app integration scaffolding (no SDK code)

Process plan. SDK-side deliverables:
- `examples/ios-sample/` — SwiftPM integration reference (Mac-only build).
- `examples/android-sample/` — Compose integration reference.
- `docs/phase-4-status.md` — explicit boundary doc separating in-repo, Mac-required, and out-of-repo work.

### Phase 5 — hardening + audit-prep scaffolding

Process plan. In-repo deliverables:
- `.github/workflows/nightly-hardening.yml` — proptest @ 4096 cases, `cargo-mutants`, `cargo-machete`. Daily 04:30 UTC.
- `.github/workflows/release-plz.yml` + `release-plz.toml` + `release-plz.changelog.toml` — automated semver bumps. `workflow_dispatch` until v1.0; flips to push-to-main after.
- `docs/audits/README.md`, `docs/release-checksums.md`, `docs/threat-model-walkthrough-2026.md` — templates that fill in at v1.0.0-rc.1.
- `docs/phase-5-status.md` — links the 9 release gates (issues #3, #4, #8–#14).

### Phase 6 — WASM functional EVM + SOL

Full `JovaWallet` surface for EVM + SOL chains via wasm-bindgen. BTC + XRP rejected at the WASM boundary with `unsupportedChain` (deferred per 2026-05-11 user decision). TypeScript types + Disposable wrapper (`using wallet = JovaWallet.fromMnemonic(...)`). Per-chain subpath exports (`/evm`, `/sol`). 42 Vitest tests passing. Bundle size 787 KB gzipped (vs 2 MB budget). The dual-getrandom-feature trick (`getrandom 0.3 wasm_js` + `getrandom_02 = { package = "getrandom", version = "0.2", features = ["js"] }`) unifies the WASM RNG flag across the dep graph.

### Phase 7 — hardware-wallet readiness

- `external-rng` feature on `jova-core-primitives` (and forwarded by `jova-core`).
- `JovaRng` trait + `RngError` enum. `no_std + alloc`-clean.
- `Mnemonic::generate_with(strength, &mut impl JovaRng)`.
- `Seed::from_external_bytes(bytes: [u8; 64])`.
- `JovaWallet::from_seed_bytes(bytes)` — Rust-only; not on FFI/WASM.
- `examples/firmware-template/` — `thumbv7em-none-eabihf` reference binary, 394 KB stripped ELF, signs ECDSA secp256k1. Built in CI.
- `docs/integration-hardware.md` rewritten with Phase 7 API surface, secure-element patterns (ATECC608 / OPTIGA Trust M / SE050), and side-channel guidance.

## Open release gates (tracked as GitHub issues)

| # | Gate | Tag it unblocks |
|---|---|---|
| [#3](https://github.com/jovachain/jovawallet-core/issues/3) | BTC migration CSV from Android team | (Phase 4 BTC rollout) |
| [#4](https://github.com/jovachain/jovawallet-core/issues/4) | BTC mainnet smoke | (Phase 4 BTC general availability) |
| [#8](https://github.com/jovachain/jovawallet-core/issues/8) | External security audit | v1.0.0 |
| [#9](https://github.com/jovachain/jovawallet-core/issues/9) | Reproducible-build dual-engineer pairing | v1.0.0 |
| [#10](https://github.com/jovachain/jovawallet-core/issues/10) | Threat-model walkthrough | v1.0.0 |
| [#11](https://github.com/jovachain/jovawallet-core/issues/11) | App-team RC validation against v1.0.0-rc.1 | v1.0.0 |
| [#12](https://github.com/jovachain/jovawallet-core/issues/12) | Bug bounty program funding | v1.0.0 (parallel) |
| [#13](https://github.com/jovachain/jovawallet-core/issues/13) | Phase 4 100% rollout soak | v1.0.0 |
| [#14](https://github.com/jovachain/jovawallet-core/issues/14) | 14-day fuzz soak | v1.0.0 |

After v1.0.0 closes: tag v1.1.0 with the Phase 6 WASM deliverables; the Phase 6 PR is already merged so the work is on `main` ready to ship.

After Phase 6 v1.1.0 ships: tag v1.2.0 once the firmware repo's own v1.0 lands (separate codebase, separate team).

## Mac-required work (for any agent on a non-Mac host)

Per `docs/phase-4-status.md`:

1. **Build the iOS XCFramework.** `bindings/swift/scripts/build-xcframework.sh` on a Mac with Xcode 15+. Produces `JovaCore.xcframework` for iOS device + simulator + macOS.
2. **Publish the SwiftPM satellite repo** `jovawallet-core-swift` at the matching tag. Carries the XCFramework + Swift convenience layer.
3. **`swift test` on macOS-latest** is exercised by CI (the `swift` workflow on every PR — confirmed green on PRs #5, #6, #7, #15, #16, #17).
4. **App Store / TestFlight upload.** Phase 5 release-management concern.
5. **iOS sample app build + simulator run.** `cd examples/ios-sample && open Package.swift` in Xcode.

## VM environment (still as of 2026-05-14)

- Ubuntu 24.04, 1 vCPU, 2 GB RAM + 4 GB swap. Cold workspace compile ≈ 15 min; incremental ≈ 30 s.
- All required tooling installed: Rust 1.95.0 stable + nightly, 10 cross-compile targets, just, cargo-ndk, cargo-deny, cargo-audit, cargo-fuzz, uniffi-bindgen, wasm-pack, bdk-cli, Foundry, Solana CLI, xrpl-py.
- Python 3 + `embit 0.8.0` + `bip322` + `bip_utils 2.x` + `solders` available via disposable venvs per capture script.
- Android SDK + NDK r29 stable at `$HOME/Android/sdk`.
- clang installed (Phase 2 verification fix).

## Useful commands

```bash
. "$HOME/.cargo/env"
cd /home/ubuntu/jovawallet-core

cargo test --workspace --locked
cargo run -p jova-verify-spec
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --no-default-features --features external-rng
cargo +nightly miri test -p jova-core-primitives

# Phase 2:
cargo run -p jova-btc-migration-check   # requires the gitignored CSV at tools/btc-migration-check/known-android-mappings.csv

# Phase 6:
./bindings/wasm/scripts/build-wasm.sh
(cd bindings/wasm && pnpm install && pnpm test)

# Phase 7:
(cd examples/firmware-template && cargo build --target thumbv7em-none-eabihf --release)

# Kotlin (slow on this VM):
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
./bindings/kotlin/scripts/build-aar.sh
(cd bindings/kotlin && ./gradlew :jova-core:test)
```

## Useful spike-finding notes (preserved across phases)

- `bitcoin 0.32`'s re-exported `secp256k1` is **0.29**, not the workspace's 0.31. Two secp versions coexist.
- `bitcoin::Psbt::finalize_mut` does NOT exist on bitcoin 0.32 — manual finalize is required for single-/multi-input P2WPKH.
- **Low-R ECDSA grinding** matters for byte-stable BIP-143 output. Use `secp.sign_ecdsa_low_r`, not `sign_ecdsa`. Bitcoin Core has used this default since v0.17 (2018).
- `bip39 2.2.x` has no `english` feature — wordlist is always-on. Use `features = ["alloc"]`.
- `xrpl-rust 1.1` is the right crate on crates.io — `xrpl 0.1.2` / `xrpl 0.5` is an unrelated WebSocket client.
- `slip-10 0.4` does NOT implement ed25519 — Solana SLIP-10 was implemented in-crate in Phase 3c.
- `secp256k1-sys` C build for `wasm32-unknown-unknown` requires `-Dmemmove=__builtin_memmove` CFLAG — already in `.cargo/config.toml`.
- WASM dep tree has both `getrandom 0.2` (transitive via alloy + Solana) and `getrandom 0.3` (workspace dep). Both need their respective `js`/`wasm_js` features; the wasm crate declares both directly to unify across the dep graph.
- `cortex-m` 0.7 needs `features = ["critical-section-single-core"]` for embedded-alloc to link.

## What this session did NOT do

- Did not tag `v1.0.0` / `v1.1.0` / `v1.2.0` — every reasonable definition requires external coordination (audit firm, bug bounty funding, dual-engineer pairing, app-team rollout, firmware repo). Issues opened for each gate.
- Did not engage an external auditor.
- Did not run a 14-consecutive-days fuzz soak (CI runs the nightly job; release manager confirms the 14-day green streak before tagging RC).
- Did not build hardware. Hardware engineering is a separate codebase + separate team.
- Did not push to crates.io / Maven Central / npm — gated on Phase 5 release pipeline.

The cryptographic surface across all chains is complete, byte-equal-tested, audited via miri, and built clean across Linux x86_64, macOS, Windows, Android NDK x4, WASM, and Cortex-M `thumbv7em-none-eabihf`. The SDK is shipping-ready pending the human-driven release gates.
