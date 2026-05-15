# HANDOFF

**Status (2026-05-16):** Phases 2 through 7 shipped. **`v0.3.1` is the latest tag on `main`**, with the matching SwiftPM satellite release at `github.com/jovachain/jovawallet-core-swift@v0.3.1`. The next milestone (`v1.0.0`) is gated on external blockers (audit, app-team rollout, bug bounty funding) tracked as GitHub issues #3, #4, #8–#14.

## Mac handoff session — 2026-05-15

Ran on macOS 26.4 / Xcode 26.5 / Rust 1.95.0 / uniffi 0.31.1. All Mac-required Phase 4 deliverables landed.

### Shipped
1. **iOS XCFramework rebuilt** for `main` at `1a8c961`. Cold build 3:36. Three slices (`ios-arm64`, `ios-arm64-simulator`, `macos-arm64_x86_64`) each verified with `lipo -info`. `swift test --parallel` from `bindings/swift/` → 16/16 green. Zipped artifact 270 MB, sha256 `506b0bb5f2bc23f72daca43f0dbada729f5506a6761569394e7382f961a39a07`.
2. **SwiftPM satellite repo** `github.com/jovachain/jovawallet-core-swift` created public, initial commit `7b57dc7`, release `v0.3.0` published with the zipped XCFramework as the lone asset. A throwaway SPM consumer (`.package(url: "…jovawallet-core-swift", from: "0.3.0")`) resolves, downloads the binary artifact, builds, and runs cleanly.
3. **iOS sample verified.** `examples/ios-sample/` compiles for macOS (host) and iOS simulator after fixing two committed source bugs (see commits in this PR). CLI smoke test exercised `JovaWallet.fromMnemonic` + `address(chain:)` + `signTx` + `signMessage` for ethereum, bitcoin, solana, xrp — all produced expected outputs.
4. **Android AAR built.** All 4 ABIs as correct ELF arches. `./gradlew :jova-core:test` → 16/16 green via JNA against the host dylib (same vectors as Swift). Android sample now compiles after fixing `WalletRepository.kt`.

### Follow-ups from this session
| Issue | Topic | Status |
|---|---|---|
| [#21](https://github.com/jovachain/jovawallet-core/issues/21) | XCFramework macOS deployment target flag | ✅ Closed by PR #23 |
| [#20](https://github.com/jovachain/jovawallet-core/issues/20) | AGP 8.5.0 vs `compileSdk = 36` | ✅ Closed by PR #24 (bumped to AGP 8.10.1) |
| [#22](https://github.com/jovachain/jovawallet-core/issues/22) | NDK strip version mismatch | ✅ Closed by PR #24 (pinned `ndkVersion = "29.0.14206865"`) |
| [#19](https://github.com/jovachain/jovawallet-core/issues/19) | Satellite tests can't locate `spec/test-vectors.json` | Open — cosmetic; satellite ships binary, not test coverage |

### Closeout — `v0.3.1` (2026-05-15)

Tagged `v0.3.1` on `main` at `10f324d` after PRs #18 and #23 merged. Rebuilt the XCFramework with the deployment-target fix (zip sha256 `6fc196dcffe5ef502c670d3cadfc2507a38fa2986d966a933c40be81cab0a5f2`) and published satellite v0.3.1 with that asset. A throwaway SPM consumer at `from: "0.3.1"` resolves, downloads the binary, and builds with **0 linker warnings** — the regression every prior consumer build saw is gone. Subsequent PR #24 closed the AGP + NDK gradle drift on `main`. Outstanding work all moves out-of-repo from here.

## What's on `main` now

```
62617dd  fix(kotlin): bump AGP to 8.10.1 + pin ndkVersion to r29 (#24)
10f324d  fix(swift): pin XCFramework deployment targets to package floors (#23)
7975f8c  feat: Mac handoff complete — XCFramework, SwiftPM satellite v0.3.0, sample fixes (#18)
1a8c961  docs: add MAC-HANDOFF.md (since retired)
ee641dd  docs: refresh HANDOFF after Phases 2-7 ship + v0.3.0 tag
57e6266  feat(primitives): Phase 7 hardware-wallet readiness (#17)
954fb1e  Phase 6: WASM functional (EVM + SOL) (#16)
581e805  chore(phase-5): hardening pipeline scaffolding + v1.0 release gates (#15)
a69c26a  docs(phase-4): example apps + Mac-required boundary document (#7)
a15ef36  Phase 3: Solana + XRP + remaining EVM (v0.3.0) (#6)
a18f3e9  Phase 2: Bitcoin (BIP-84 + PSBT + BIP-322) — v0.2.0 (#5)
2d1aa12  Phase 1: EVM end-to-end (v0.1.0) (#2)
e07107f  Phase 0: repo bootstrap (PR #1)
```

Tags: `v0.0.1`, `v0.1.0`, `v0.2.0`, `v0.3.0`, `v0.3.1`. The next tag (`v1.0.0` from Phase 5 completion) is the audit / RC / bug-bounty milestone.

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

## Mac-required work — complete as of `v0.3.1`

| Item | Status |
|---|---|
| Build the iOS XCFramework | ✅ Shipped in `v0.3.1` via `bindings/swift/scripts/build-xcframework.sh` (deployment targets pinned to `MACOSX_DEPLOYMENT_TARGET=11.0` and `IPHONEOS_DEPLOYMENT_TARGET=14.0`) |
| Publish SwiftPM satellite repo | ✅ `github.com/jovachain/jovawallet-core-swift@v0.3.1` published; throwaway SPM consumer verified — 0 linker warnings |
| `swift test` on macOS-latest | ✅ Green on every PR via the `swift` CI workflow |
| iOS sample app build | ✅ Compiles for macOS host + iOS simulator slice; runtime smoke covers BTC, EVM, SOL, XRP |
| App Store / TestFlight upload | Phase 5 release-management concern (app team, not SDK) |

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
