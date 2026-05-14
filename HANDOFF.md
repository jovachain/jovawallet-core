# HANDOFF — autonomous session (2026-05-14)

This file is the post-execution summary for whoever picks up after this Linux VM session. **Read this AND `AGENT-README.md` AND `CLAUDE.md` before doing anything else.**

## Where we are right now

- **Branch:** `feat/phase-2-bitcoin` (pushed to origin)
- **Phase 2 status:** Tasks 1 and 2 of 12 done, committed, pushed. See "Phase 2 task tracker" below.
- **PR status:** No PR opened yet — branch lives on its own. Open one once Tasks 6, 7, 8, 9 are also done (Tasks 10 and 11 are external blockers — see below).
- **Tag status:** Unchanged from handoff. `v0.0.1`, `v0.1.0` on `main`. No new tags.

## Decisions made autonomously this session

1. **Git identity** set to `xhuman <xhuman.77x@gmail.com>` per user instruction. Replaces `jovachain-agent <agent@jovachain.local>`. Phase 2 commits will show this author; Phase 0/1 history is untouched.
2. **`gh` CLI** authenticated with a fine-grained PAT scoped to the `jovachain` GH account. The token was provided in-session; if you need to re-auth, ask the user for a fresh token.
3. **CHANGELOG.md** backfilled with real `[0.0.1]` (2026-05-12) and `[0.1.0]` (2026-05-13) entries plus a working `[Unreleased]` section for Phase 2. Was a flagged "small debt" in the handoff — now current.
4. **No autonomous architectural deviations.** All commits follow the locked conventions: `#![forbid(unsafe_code)]`, no_std-clean primitives, engine confinement (`bitcoin` only inside `jova-core-chains`), Conventional Commits, no AI attribution anywhere.

## VM environment

- Ubuntu 24.04, **1 vCPU, 2 GB RAM + 4 GB swap.** Cold workspace compile = **~16 min**. Plan accordingly.
- All required cargo tools are installed: `just`, `cargo-ndk`, `cargo-deny`, `cargo-audit`, `cargo-fuzz`, `uniffi-bindgen`, `wasm-pack`, `bdk-cli`.
- All external reference signers installed: Foundry (`cast`, `forge`, `anvil`), Solana CLI (Anza). `xrpl-py` was pre-installed via `pipx`.
- Rust toolchain: 1.95.0 stable + nightly. All 10 cross-compile targets installed.
- Android SDK + NDK r29 stable (`29.0.14206865`) at `$HOME/Android/sdk` — verify with `ls $ANDROID_NDK_HOME` before building Kotlin.

## Phase 2 task tracker

| # | Task | State | Notes |
|---|---|---|---|
| 1 | BIP-84 derivation helper | ✅ done | commit `6d764a2` — 9 tests pass (3 official BIP-84 pubkeys byte-identical) |
| 2 | P2WPKH (bech32) address derivation | ✅ done | commit `63e5097` — 9 tests pass (3 official BIP-84 addresses) |
| 3 | PSBT signing — single-input | ⏳ TODO | Needs vector capture via `bdk-cli` on regtest. Use `tools/btc-vector-capture/single_input.sh` (script does not yet exist; create per plan §3) |
| 4 | PSBT — multi-input + multi-party | ⏳ TODO | Two capture scripts (`multi_input.sh`, `multi_party.sh`). Multi-party returns `psbt:` prefix in `raw_hex` to signal unfinalized state |
| 5 | BIP-322 + legacy `signMessage` | ⏳ TODO | Capture from `bdk-cli sign_message --scheme {bip322,legacy}` |
| 6 | `BtcSigner` trait impl + `JovaWallet` dispatch | ⏳ TODO | Pure Rust. Adds `UnsignedTx::Bitcoin { psbt_base64 }`, `SignableMessage::Bitcoin { … }`, `JovaChain::Bitcoin`. Existing EVM match arms need wildcards added or new arms (currently exhaustive on `UnsignedTx::Evm(_)`) |
| 7 | 12 BTC vectors in `spec/test-vectors.json` | ⏳ TODO | Bump `version` to `"0.3"`. `tools/verify-spec` already rejects `TODO`/`<capture>`/`REPLACE` placeholders |
| 8 | Vector parity Rust + Swift + Kotlin | ⏳ TODO | Swift parity is CI-only (`macos-latest` runner). Kotlin parity runs locally via JNA + Gradle |
| 9 | Property tests + 3 fuzz targets | ⏳ TODO | `fuzz_psbt_sign`, `fuzz_btc_address_parse`, `fuzz_bip322_verify`. Update `fuzz/Cargo.toml` `[[bin]]` blocks and `justfile`'s `fuzz` recipe |
| 10 | Migration spot-check (100/100) | 🛑 **BLOCKED** | `tools/btc-migration-check/known-android-mappings.csv` does **not** exist in the repo. Android team must export it. Until it's present, this task is not runnable |
| 11 | Mainnet smoke test | 🛑 **BLOCKED** | Needs human + real BTC funds. Cannot be done autonomously. Document the tx hash in `docs/btc-mainnet-smoke.md` after manual run |
| 12 | PR + CI + tag `v0.2.0` | ⏳ TODO | All 6 CI workflows must be green. Strip the `🤖 Generated with Claude Code` line that's in the plan's PR template — **the repo policy is NO AI attribution.** |

## Phases 3 — 7 status

All still pending. See `docs/superpowers/plans/2026-05-05-phase-N-*.md` for each plan. Recap:

- **Phase 3 → `v0.3.0`** — Solana (v0 versioned txs via Anza split crates), XRP (`xrpl-rust 1.1`, not the unrelated `xrpl` crate), and the remaining EVM chains (Arbitrum, Optimism, Base, customEvm). XRP vector capture uses `xrpl-py` via pipx (already installed). Solana uses `solana-cli` (installed).
- **Phase 4 → milestone (no tag)** — wallet-app integration, partly out-of-repo. Process plan.
- **Phase 5 → `v1.0.0`** — hardening, audit prep, cargo-vet wired into CI, RC cycles. External audit is human-coordinated.
- **Phase 6 → `v1.1.0`** — WASM functional, **EVM + SOL only** (BTC/XRP browser signing deferred per recorded user decision 2026-05-11). `secp256k1-sys` C build for `wasm32-unknown-unknown` requires `-Dmemmove=__builtin_memmove` CFLAG, already in `.cargo/config.toml`.
- **Phase 7** — `no_std` primitives audit on `thumbv7em-none-eabihf` (the no-std build already runs in CI). Embedded benches and firmware-target proof.

## Hard blockers the next agent should surface to the user

1. **Phase 2 Task 10:** CSV file `tools/btc-migration-check/known-android-mappings.csv` from the Android team. Without it, migration spot-check is not runnable. The directory's `.gitignore` already excludes it from commits — user mnemonics must NOT be committed.
2. **Phase 2 Task 11:** Mainnet smoke needs ~$5 of BTC + a manual broadcast. The engineer driving the phase performs this.
3. **Phase 4:** Most work happens in the wallet-app repos, not here. Coordinate with the iOS/Android teams.
4. **Phase 5:** External audit firm needs to be engaged (3-4 week lead time).

## Useful local commands

```bash
. "$HOME/.cargo/env"
export PATH="$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
cd /home/ubuntu/jovawallet-core

# Per-task TDD:
cargo test -p jova-core-primitives --test bip84
cargo test -p jova-core-chains --test btc_address

# Pre-PR gauntlet (slow on this VM):
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

Swift parity is CI-only (`macos-latest` runner). Mirror the Phase 1 pattern in `bindings/swift/Tests/JovaCoreTests/EvmVectorsTests.swift` for each new vector kind.

## Do-not-litigate decisions (recap)

- WASM scope: BTC/XRP browser signing deferred beyond v1.1.
- Conventional Commits from Phase 0 onward.
- No AI attribution anywhere in commits, PR bodies, or files.
- Vectors come from external reference signers (`cast`, `bdk-cli`, `solana-cli`, `xrpl-py`); Rust code matches the captured vector byte-for-byte.
- Engine confinement: `bdk_wallet`, `alloy`, `bitcoin`, `solana-*`, `xrpl-rust` only inside `crates/jova-core-chains`.

## What this session did NOT do

- Did not open a PR (waiting for more substantive Phase 2 progress).
- Did not capture any PSBT or BIP-322 vectors (Tasks 3–5 not yet attempted in this update of HANDOFF.md — will update as work progresses).
- Did not touch Phase 3+ code or plans.
- Did not run the full pre-PR gauntlet (waiting until Phase 2 is closer to ship-ready).
