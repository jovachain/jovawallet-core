# HANDOFF — autonomous session (2026-05-14)

This file is the post-execution summary for whoever picks up after this Linux VM session. **Read this AND `AGENT-README.md` AND `CLAUDE.md` before doing anything else.**

## Where we are right now

- **Branch:** `feat/phase-2-bitcoin` (pushed to origin)
- **Phase 2 status:** Tasks 1 + 2 of 12 done. Repo builds clean. Tests pass.
- **PR status:** No PR opened. Open one after Tasks 6, 7, 8, 9 are also done. Tasks 10 and 11 are external blockers — see "Hard blockers" below.
- **Tag status:** Unchanged. `v0.0.1`, `v0.1.0` on `main`. No new tags.

## Commits on `feat/phase-2-bitcoin`

| SHA | Message |
|---|---|
| `6d764a2` | feat(primitives): BIP-84 derivation path helper + BIP-84 official vectors |
| `63e5097` | feat(chains/btc): P2WPKH address derivation + bech32 validation |
| `fd071f8` | docs: backfill CHANGELOG entries for v0.0.1 + v0.1.0; add HANDOFF.md |

`cargo build --workspace` finishes in ~38 s incremental on this VM after the bitcoin crate is compiled once (cold compile of the bitcoin dep tree is ~15 min).

## Decisions made autonomously this session

1. **Git identity** set to `xhuman <xhuman.77x@gmail.com>` per user instruction. Replaces `jovachain-agent <agent@jovachain.local>` going forward; Phase 0/1 history is untouched.
2. **`gh` CLI** authenticated with a fine-grained PAT scoped to the `jovachain` GH account. Token was provided in-session. Re-auth with a fresh token if needed.
3. **CHANGELOG.md** backfilled with real `[0.0.1]` (2026-05-12) and `[0.1.0]` (2026-05-13) entries plus a working `[Unreleased]` section for Phase 2. Was a flagged "small debt" in the handoff — now current.
4. **`bitcoin` 0.32** added to `jova-core-chains` workspace dep inheritance with `features = ["std"]`. The workspace `bitcoin` dep is declared with `default-features = false, features = ["secp-recovery"]`; crate-local `std` is required because chains is a std crate. `bdk_wallet` is **not** added yet — defer until the PSBT-signing task actually uses it.
5. **No autonomous architectural deviations.** All commits follow the locked conventions: `#![forbid(unsafe_code)]`, no_std-clean primitives, engine confinement (`bitcoin` only inside `jova-core-chains`), Conventional Commits, no AI attribution anywhere.

## VM environment (as of 2026-05-14)

- Ubuntu 24.04, **1 vCPU, 2 GB RAM + 4 GB swap.** Cold workspace compile of new crate deps = **~15 min**. Plan TDD cycles accordingly.
- All required cargo tools installed: `just`, `cargo-ndk`, `cargo-deny`, `cargo-audit`, `cargo-fuzz`, `uniffi-bindgen`, `wasm-pack`, `bdk-cli`.
- External reference signers installed: Foundry (`cast`, `forge`, `anvil`), Solana CLI (Anza). `xrpl-py` was pre-installed via `pipx`.
- Rust toolchain: 1.95.0 stable + nightly. All 10 cross-compile targets installed.
- Android SDK + NDK r29 stable (`29.0.14206865`) at `$HOME/Android/sdk`.

## Phase 2 task tracker

| # | Task | State | Notes |
|---|---|---|---|
| 1 | BIP-84 derivation helper | ✅ done | commit `6d764a2` — 9 tests pass (3 BIP-84 official pubkeys byte-identical) |
| 2 | P2WPKH (bech32) address derivation | ✅ done | commit `63e5097` — 9 tests pass (3 BIP-84 official addresses, rejection of P2PKH, P2SH, Taproot, testnet) |
| 3 | PSBT signing — single-input | ⏳ TODO | Capture vectors via `bdk-cli` on regtest first. Create `tools/btc-vector-capture/single_input.sh` per plan §3 |
| 4 | PSBT — multi-input + multi-party | ⏳ TODO | Multi-party returns `psbt:` prefix in `raw_hex` to signal unfinalized. Multi-input owns all keys — finalizes. |
| 5 | BIP-322 + legacy `signMessage` | ⏳ TODO | Capture from `bdk-cli sign_message --scheme {bip322,legacy}` |
| 6 | `BtcSigner` trait impl + `JovaWallet` dispatch | ⏳ TODO | Adds `UnsignedTx::Bitcoin { psbt_base64 }`, `SignableMessage::Bitcoin { … }`, `JovaChain::Bitcoin` and routes through new sign_psbt / sign_btc_message |
| 7 | 12 BTC vectors in `spec/test-vectors.json` | ⏳ TODO | Bump `version` to `"0.3"`. `tools/verify-spec` already rejects `TODO`/`<capture>`/`REPLACE` placeholders |
| 8 | Vector parity Rust + Swift + Kotlin | ⏳ TODO | Swift parity is CI-only (`macos-latest` runner). Kotlin parity runs locally via JNA + Gradle |
| 9 | Property tests + 3 fuzz targets | ⏳ TODO | `fuzz_psbt_sign`, `fuzz_btc_address_parse`, `fuzz_bip322_verify`. Update `fuzz/Cargo.toml` `[[bin]]` blocks and `justfile`'s `fuzz` recipe |
| 10 | Migration spot-check (100/100) | 🛑 **BLOCKED** | `tools/btc-migration-check/known-android-mappings.csv` does **not** exist in the repo. Android team must export it |
| 11 | Mainnet smoke test | 🛑 **BLOCKED** | Needs human + real BTC funds. Cannot be done autonomously |
| 12 | PR + CI + tag `v0.2.0` | ⏳ TODO | Strip the `🤖 Generated with Claude Code` line from the plan's PR-body template — repo policy is **NO AI attribution** anywhere |

## Bitcoin 0.32 API gotchas I hit (so you don't repeat them)

I attempted Tasks 3+5+6 in this session and reverted before commit because the bitcoin 0.32 / secp256k1 0.31 API differs from the plan's snippets. Capture these in your next pass:

1. **`bitcoin::hashes::Hash` trait is not in scope by default.** `to_byte_array()` and `as_byte_array()` are trait methods. Add `use bitcoin::hashes::Hash;` at the top of any module that handles `WPubkeyHash`, `SegwitV0Sighash`, `sha256::Hash`, etc.
2. **`secp256k1::ecdsa::Signature::normalize_s(&mut self)` returns `()` in 0.31.** Mutates in place — don't write `let sig = secp.sign_ecdsa(...).normalize_s();`. Write `let mut sig = secp.sign_ecdsa(...); sig.normalize_s();`.
3. **`MessageSignature::to_base64()` does not exist.** Check the current `bitcoin::sign_message` module API. You may need to construct base64 manually or use a different method name.
4. **`base64` is not a dep of `jova-core-chains` yet.** It's declared in workspace deps but not inherited by chains. Add `base64.workspace = true` to `crates/jova-core-chains/Cargo.toml` when you need it.
5. **`bdk_wallet` 3.0** is in workspace deps but not used yet by any crate. Inherit it into chains when you start the PSBT work. Its API for `Psbt::finalize_mut` may have moved between minor versions; trust the cargo error messages over the plan snippets.
6. **`secp256k1::Signature::serialize_der()` exists** but is on the signature value itself, not the result of `normalize_s()`. Pattern: `let sig = secp.sign_ecdsa(&msg, &sk); sig.normalize_s(); sig.serialize_der()`.

The plan snippets in `docs/superpowers/plans/2026-05-05-phase-2-bitcoin.md` describe an older bitcoin API. **Trust the captured test vectors as the contract**, adjust function calls to whatever the current crate uses.

## Phases 3 — 7 status

All still pending. See `docs/superpowers/plans/2026-05-05-phase-N-*.md`. Recap of the major dependency facts:

- **Phase 3 → `v0.3.0`** — Solana (v0 versioned txs via Anza split crates), XRP (`xrpl-rust 1.1`, **not** the unrelated `xrpl` crate on crates.io), and the remaining EVM chains (Arbitrum, Optimism, Base, customEvm — the variants are already in `JovaChain`). XRP vector capture uses `xrpl-py` via pipx (installed). Solana uses `solana-cli` (installed at `~/.local/share/solana/install/active_release/bin/solana`).
- **Phase 4 → milestone (no tag)** — wallet-app integration, partly out-of-repo. Process plan.
- **Phase 5 → `v1.0.0`** — hardening, audit prep, cargo-vet wired into CI, RC cycles. External audit is human-coordinated (3-4 week lead time).
- **Phase 6 → `v1.1.0`** — WASM functional, **EVM + SOL only** (BTC/XRP browser signing deferred per recorded user decision 2026-05-11). `secp256k1-sys` C build for `wasm32-unknown-unknown` requires `-Dmemmove=__builtin_memmove` CFLAG, already in `.cargo/config.toml`.
- **Phase 7** — `no_std` primitives audit on `thumbv7em-none-eabihf` (the no-std build already runs in CI). Embedded benches and firmware-target proof.

## Hard blockers the next agent should surface to the user

1. **Phase 2 Task 10:** CSV file `tools/btc-migration-check/known-android-mappings.csv` from the Android team. Without it, migration spot-check is not runnable. The directory's `.gitignore` already excludes it from commits — user mnemonics must NOT be committed.
2. **Phase 2 Task 11:** Mainnet smoke needs ~$5 of BTC + a manual broadcast. The engineer driving the phase performs this.
3. **Phase 4:** Most work happens in the wallet-app repos, not here. Coordinate with the iOS/Android teams.
4. **Phase 5:** External audit firm needs to be engaged (3-4 week lead time).

## Suggested execution order for the next agent

1. Resume on `feat/phase-2-bitcoin` (already pushed).
2. Capture PSBT single-input vector via `bdk-cli` on regtest → drop captures into `tools/btc-vector-capture/captures/*` → write the failing test → implement `sign_psbt` (Task 3) → commit → push.
3. Capture multi-input + multi-party vectors → tests pass without code change → commit (Task 4) → push.
4. Capture BIP-322 + legacy vectors → write tests → implement `sign_btc_message` (Task 5) → commit → push.
5. Add `BtcSigner` trait impl + `JovaWallet` dispatch + the `Bitcoin` variants on `UnsignedTx` / `SignableMessage` / `JovaChain` + FFI mapping updates (Task 6) → commit → push.
6. Author 12 BTC vectors in `spec/test-vectors.json` (Task 7).
7. Rust + Swift + Kotlin vector parity (Task 8).
8. Property tests + 3 fuzz targets (Task 9).
9. Run the pre-PR gauntlet (commands below).
10. Open PR for Phase 2. **Strip AI attribution.** Wait for all 6 CI workflows green, squash-merge, tag `v0.2.0`.
11. Move to Phase 3.

## Useful local commands

```bash
. "$HOME/.cargo/env"
export PATH="$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
cd /home/ubuntu/jovawallet-core

# Per-task TDD examples (these passed locally):
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

## Launching a new session with permission bypass

The next session should be started with:

```bash
claude --dangerously-skip-permissions
```

That skips all tool-permission prompts. The first agent in this session was launched with that flag already, so all the cargo installs, file edits, and `git push` operations went through without confirmation.

## Do-not-litigate decisions (recap)

- WASM scope: BTC/XRP browser signing deferred beyond v1.1.
- Conventional Commits from Phase 0 onward.
- No AI attribution anywhere in commits, PR bodies, or files.
- Vectors come from external reference signers (`cast`, `bdk-cli`, `solana-cli`, `xrpl-py`); Rust code matches the captured vector byte-for-byte.
- Engine confinement: `bdk_wallet`, `alloy`, `bitcoin`, `solana-*`, `xrpl-rust` only inside `crates/jova-core-chains`.

## What this session did NOT do

- Did not open a PR (waiting for more substantive Phase 2 progress — Tasks 3-9 still ahead).
- Did not capture any PSBT or BIP-322 vectors.
- Did not finish wiring `UnsignedTx::Bitcoin` / `SignableMessage::Bitcoin` / `JovaChain::Bitcoin` — attempted Tasks 3+5+6 in one batch, hit bitcoin 0.32 API mismatches, reverted to keep the repo green. See "Bitcoin 0.32 API gotchas" above.
- Did not touch Phase 3+ code or plans.
- Did not run the full pre-PR gauntlet.
