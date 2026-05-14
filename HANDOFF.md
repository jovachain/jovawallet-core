# HANDOFF — autonomous session (2026-05-14)

This file is the post-execution summary for whoever picks up next. **Read this AND `AGENT-README.md` AND `CLAUDE.md` before doing anything else.**

## Where we are right now

- **Branch:** `feat/phase-2-bitcoin` (pushed, ready for PR).
- **Phase 2 status:** Tasks 1-9 complete. Tasks 10-11 are external blockers tracked as GitHub issues #3 and #4. Task 12 (pre-PR gauntlet + open PR + merge + tag `v0.2.0`) is the next step.
- **PR status:** No PR opened yet. Open after the local pre-PR gauntlet passes.
- **Tag status:** Unchanged. `v0.0.1`, `v0.1.0` on `main`. No new tags.

## Commits on `feat/phase-2-bitcoin`

| SHA | Message |
|---|---|
| `6d764a2` | feat(primitives): BIP-84 derivation path helper + BIP-84 official vectors |
| `63e5097` | feat(chains/btc): P2WPKH address derivation + bech32 validation |
| `fd071f8` | docs: backfill CHANGELOG entries for v0.0.1 + v0.1.0; add HANDOFF.md |
| `a8b7355` | docs: update HANDOFF with bitcoin 0.32 API gotchas and revert note |
| `233409b` | feat(chains/btc): single-input PSBT signing |
| `895918e` | refactor(chains/btc): tighten PSBT signing per code review |
| `46d0d52` | feat(chains/btc): multi-input + multi-party PSBT signing |
| `c98fc6b` | feat(chains/btc): BIP-322 + legacy signMessage |
| `8c29451` | feat(core): JovaWallet dispatch to BtcSigner via JovaChain::Bitcoin |
| `b27aeef` | feat(spec): 12 BTC vectors covering BIP-84, PSBT, BIP-322, error paths |
| `acba201` | test: BTC vector parity on Rust + Kotlin (+ Swift CI-only) |
| `8487fae` | test(btc): property tests + 3 fuzz targets |

## Decisions made autonomously this session (Phase 2)

1. **Capture path:** all BTC vector captures use `embit 0.8.0` Python as the independent reference signer (RFC-6979 deterministic ECDSA + low-R grinding, matching Bitcoin Core ≥ v0.17 defaults). Bitcoind regtest + bdk-cli (the plan's path A) was rejected because `bitcoind` is not in Ubuntu 24.04 apt repos and bdk-cli 3.0 requires a configured backend. The captures are reproducible: capture scripts live in `tools/btc-vector-capture/*.sh`. BIP-322 cross-validated by the `bip322` PyPI package.
2. **bdk_wallet not pulled.** Manual P2WPKH PSBT finalization in pure `bitcoin 0.32` (witness `[sig_der || sighash_byte, compressed_pubkey]`). Avoids 15+ min cold compile of bdk_wallet's dep tree.
3. **Low-R grinding on PSBT signing** (`secp.sign_ecdsa_low_r`, not `sign_ecdsa`). The Task 3 single-input test had been passing by coincidence (its sighash happened to be low-R unconditionally); the multi-input vector capture exposed the divergence. Bitcoin Core has grinded for low-R by default since v0.17 (2018) — matching is correct product behavior. Saves ~0.5 sat/byte on every SegWit-spending tx.
4. **Multi-party PSBT signaling convention:** `SignedTx.raw_hex` is prefixed with `psbt:` when the wallet returns an unfinalized PSBT (i.e., needs another signer). Apps inspect the prefix to decide broadcast vs. hand-off. `tx_hash` is empty for multi-party. Documented in `crates/jova-core-chains/src/btc/mod.rs::BtcSigner::sign_tx`.
5. **BIP-322 implemented in-tree** against `bitcoin 0.32` primitives (tagged hash + virtual to_spend/to_sign txs + BIP-143 sighash + low-R ECDSA + witness consensus base64). No external `bip322` Rust crate pulled in. Output format: bare base64 of the witness (no `smp` BIP-322 prefix variant — that's only for verifier-disambiguation; mainstream signers like Sparrow / Unisat / ord / Leather / bdk all produce/accept the bare form).
6. **Legacy signMessage**: classic Bitcoin Core `signmessage` scheme — `"\x18Bitcoin Signed Message:\n<varint><msg>"` digest, `sign_ecdsa_recoverable` (no low-R grinding, matching Bitcoin Core's recoverable path), 65-byte `[header || r || s]` blob, base64-encoded. Header byte = `recovery_id + 31` for compressed keys.
7. **`JovaChain::Bitcoin` API gap:** `JovaWallet::address(chain, _account)` currently ignores its `account` argument. The vector `btc.address.bip84_abandon_account0_index1` exists in the spec (recording the BIP-84 official second-address vector) but is filtered out by `vectors_btc.rs::btc_address_vectors` until the API grows an index argument (Phase 3+).
8. **`tools/btc-migration-check`** scaffolded but gated on the CSV. Build succeeds; running with no CSV exits 2 with a helpful error. Will run end-to-end the moment the Android team drops `known-android-mappings.csv` into the directory. CSV is gitignored.
9. **No AI attribution anywhere** in commits, source comments, or docs. Repo policy enforced.

## VM environment (still as of 2026-05-14)

- Ubuntu 24.04, **1 vCPU, 2 GB RAM + 4 GB swap.** Cold workspace compile = 38 s incremental, 15 min from scratch. Kotlin AAR cross-compile to 4 Android targets is 20-30 min.
- All required cargo tools installed: `just`, `cargo-ndk`, `cargo-deny`, `cargo-audit`, `cargo-fuzz`, `uniffi-bindgen`, `wasm-pack`, `bdk-cli 3.0`.
- External reference signers installed: Foundry (`cast`, `forge`, `anvil`), Solana CLI (Anza), `xrpl-py` (pipx). Python 3 + `embit 0.8.0` + `bip322` available in disposable venvs per capture script.
- Rust toolchain: 1.95.0 stable + nightly. All 10 cross-compile targets installed.
- Android SDK + NDK r29 stable (`29.0.14206865`) at `$HOME/Android/sdk`.

## Phase 2 task tracker

| # | Task | State | Commit / Notes |
|---|---|---|---|
| 1 | BIP-84 derivation helper | ✅ done | `6d764a2` |
| 2 | P2WPKH (bech32) address derivation | ✅ done | `63e5097` |
| 3 | PSBT signing — single-input | ✅ done | `233409b` + `895918e` (review fix) |
| 4 | PSBT — multi-input + multi-party | ✅ done | `46d0d52` (multi-input finalize + low-R fix) |
| 5 | BIP-322 + legacy `signMessage` | ✅ done | `c98fc6b` |
| 6 | `BtcSigner` trait impl + `JovaWallet` dispatch | ✅ done | `8c29451` |
| 7 | 12 BTC vectors in `spec/test-vectors.json` | ✅ done | `b27aeef` |
| 8 | Vector parity Rust + Swift + Kotlin | ✅ done | `acba201` (Swift CI-only; Kotlin source in place, exercised by CI + Task 12 gauntlet) |
| 9 | Property tests + 3 fuzz targets | ✅ done | `8487fae` |
| 10 | Migration spot-check (100/100) | 🛑 **BLOCKED** | Tracking issue: [#3](https://github.com/jovachain/jovawallet-core/issues/3). Tool scaffolded; awaits production CSV from Android team. `docs/btc-migration-check.md` documents the gate. |
| 11 | Mainnet smoke test | 🛑 **BLOCKED** | Tracking issue: [#4](https://github.com/jovachain/jovawallet-core/issues/4). `docs/btc-mainnet-smoke.md` documents the gate. |
| 12 | Pre-PR gauntlet + PR + tag `v0.2.0` | ⏳ NEXT | Strip the `🤖 Generated with Claude Code` line from the plan's PR-body template — repo policy is **NO AI attribution** anywhere |

## Pre-PR gauntlet (run before opening the PR for Phase 2)

```bash
. "$HOME/.cargo/env"
export PATH="$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
cd /home/ubuntu/jovawallet-core

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo run -p jova-verify-spec
cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --release --no-default-features
cargo +nightly miri test -p jova-core-primitives
cargo deny check
cargo audit
./bindings/kotlin/scripts/build-aar.sh
( cd bindings/kotlin && ./gradlew :jova-core:test --console=plain )
./bindings/wasm/scripts/build-wasm.sh
( cd bindings/wasm && pnpm install && pnpm test )
```

Swift parity is CI-only on `macos-latest`.

## Phases 3 — 7 status

All still pending. See `docs/superpowers/plans/2026-05-05-phase-N-*.md`.

- **Phase 3 → `v0.3.0`** — Solana (v0 versioned txs via Anza split crates), XRP (`xrpl-rust 1.1`, **not** the unrelated `xrpl` crate on crates.io), and the remaining EVM chains (Arbitrum, Optimism, Base, customEvm — the variants are already in `JovaChain`).
- **Phase 4 → milestone (no tag)** — wallet-app integration, partly out-of-repo.
- **Phase 5 → `v1.0.0`** — hardening, audit prep, cargo-vet wired into CI, RC cycles. External audit is human-coordinated.
- **Phase 6 → `v1.1.0`** — WASM functional, **EVM + SOL only** (BTC/XRP browser signing deferred per recorded user decision 2026-05-11).
- **Phase 7** — `no_std` primitives audit on `thumbv7em-none-eabihf`.

## Hard blockers the next agent should surface to the user

1. **Phase 2 Task 10:** CSV file from Android team. [Issue #3](https://github.com/jovachain/jovawallet-core/issues/3).
2. **Phase 2 Task 11:** Mainnet smoke. [Issue #4](https://github.com/jovachain/jovawallet-core/issues/4).
3. **Phase 4:** Most work happens in the wallet-app repos (iOS/Android team).
4. **Phase 5:** External audit firm needs to be engaged (3-4 week lead time).
5. **Mac-only work (any phase):** Swift `swift test` / iOS XCFramework / App Store submission. CI runs Swift parity on `macos-latest`; deep iOS work needs a Mac dev machine.

## Bitcoin 0.32 / secp256k1 API gotchas (learned this session — for future BTC work)

1. **`bitcoin::hashes::Hash` trait** needed in scope for `.to_byte_array()` / `.as_byte_array()`.
2. **`secp.sign_ecdsa(...).normalize_s()`** doesn't chain — `normalize_s` mutates in place and returns `()`. Bind with `let mut sig = secp.sign_ecdsa(...); sig.normalize_s();`.
3. **Low-R grinding** matters for byte-stable BIP-143 output. Use `secp.sign_ecdsa_low_r`, not `sign_ecdsa`.
4. **`bitcoin 0.32`'s re-exported `secp256k1` is version 0.29**, NOT the workspace's 0.31. Two secp versions coexist in the build. Keep API boundaries pure-bytes (`XPrv::private_key_bytes()`, `public_key_compressed()`) to avoid leaking the version mismatch.
5. **`bitcoin::Psbt::finalize_mut` does NOT exist** on bitcoin 0.32 — that's in `miniscript::psbt::PsbtExt`. For P2WPKH single-party, finalize manually: clear `partial_sigs`, set `final_script_witness = Witness::from_slice(&[sig_der_with_sighash_byte, compressed_pk])`, call `psbt.extract_tx_unchecked_fee_rate()`.
6. **`bitcoin::ecdsa::Signature::to_vec()`** returns `sig_der || sighash_byte` in one shot. Convenient for witness construction.

## Open GitHub issues opened this session

- **#3**: Phase 2 Task 10 — CSV file gating BTC migration spot-check.
- **#4**: Phase 2 Task 11 — mainnet smoke test.

## Useful local commands

```bash
. "$HOME/.cargo/env"
cd /home/ubuntu/jovawallet-core

cargo test -p jova-core-chains
cargo test -p jova-core --test vectors_btc       # 9 tests (5 dispatch + 4 vector loops)
cargo test -p jova-core --test properties_btc    # 6 proptests
cargo run -p jova-verify-spec                    # spec drift + placeholder check
cargo run -p jova-btc-migration-check            # requires the CSV
```

## What this session did NOT do

- Did not open a PR (Task 12). Pre-PR gauntlet remains to be run, then PR + CI green + squash-merge + tag `v0.2.0`.
- Did not run the Kotlin AAR build (Task 12 gauntlet does this; ~20-30 min on this VM).
- Did not run the fuzzers for 60s each (slow VM; CI does this).
- Did not touch Phase 3+ code or plans.
