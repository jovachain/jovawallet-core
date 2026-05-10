# Phased Build Plan

The roadmap from empty repo to v1.0 and beyond. Each phase ends in a tagged release or an integration milestone in one of the apps. Time estimates are honest ranges for a senior engineer with Rust + mobile experience; less-experienced teams should add 50–100% buffer.

## Phase ordering

The phases are ordered by **risk-first** rather than craft-comfort:

1. **Feasibility spike before bootstrap** — kill toolchain dragons before writing real code.
2. **EVM first** — proves the contract end-to-end on the simplest signing path.
3. **Bitcoin second** — highest funds-on-chain risk and migration risk; do it while attention is fresh.
4. **Solana + XRP** — narrower in scope and lower funds-at-risk than BTC.
5. **App integration before audit** — feature-flagged migration so the audit reflects actual app-side wiring.
6. **Hardening + audit + v1.0** — only after every native binding has been exercised by real apps.
7. **WASM** — separate; a chain crate fighting WASM should not block the v1.0 native release.
8. **Hardware** — last; depends on hardware existing.

Backend Rust direct (`cargo add jova-core`) is **available continuously starting at Phase 1's tag** — it's a side effect of every release, not its own phase.

---

## Phase -1 — Feasibility spike (3–5 days)

**Goal:** Prove the toolchain works end-to-end before committing to writing any real chain code. This is where the dragons are.

### What this phase de-risks

- `uniffi-rs` end-to-end: Rust → Swift XCFramework + Kotlin AAR.
- `wasm-bindgen` for the same skeleton.
- The candidate dependency stack actually compiling for every target:
  - Anza's Solana split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`) on iOS / Android / WASM.
  - `bdk_wallet` on iOS / Android / WASM (tokio gating in 2026 is the historic concern).
  - `xrpl-rust` on iOS / Android / WASM (younger crate, unproven WASM track record).
  - `alloy` on every target (low-risk, still verify).
  - `secp256k1`, `ed25519-dalek`, `bip39`, `slip-10`, `zeroize` — all expected clean.

### Deliverables

- A throwaway branch `spike/feasibility` containing:
  - A trivial `lib.rs` with one exported function (`fn ping() -> String { "pong".into() }`) wrapped through every binding.
  - A bare `Cargo.toml` listing **every chain dep** as a workspace dependency (even if unused) so we link them.
  - Each binding scaffold compiles and the `ping()` round-trip works.
- A short report — `docs/feasibility-report.md` — saying: which deps compile cleanly on which targets, which need feature flags, which need to be replaced, which are blockers.

### Exit criteria

- Swift XCFramework builds on macOS CI runner, the test imports `JovaCore` and calls `ping()`.
- Kotlin AAR builds on Linux CI runner with `cargo-ndk`, JUnit imports it and calls `ping()`.
- WASM build via `wasm-pack` succeeds; vitest imports the package and calls `ping()`.
- All chain crates compile-link to at least the `aarch64-apple-ios`, `aarch64-linux-android`, and `wasm32-unknown-unknown` targets — *or* we have a documented decision about which crate gets a target-specific exclusion.

### What if a chain crate fights us

If the Solana split crates can't compile to WASM, decide now: do our own ed25519+bincode-shaped signing, or accept that SOL ships natively only and the WASM build excludes it. Decide before writing real code, not after.

The throwaway branch is *throwaway*. Phase 0 starts from a clean slate, applying lessons learned.

---

## Phase 0 — Repo bootstrap (3–5 days)

**Goal:** Empty repo with the Rust workspace building, every binding's CI green on a hello-world test, every target's compile smoke test running on every PR.

### Deliverables

- GitHub repo `jovachain/jovawallet-core`, MIT license, `README.md` linking to `docs/`.
- Satellite repo `jovachain/jovawallet-core-swift`, also MIT, empty `Package.swift` placeholder.
- `rust-toolchain.toml` pinning Rust 1.75+ stable.
- `Cargo.toml` workspace root with `[workspace.dependencies]` declared per the spike's findings.
- `crates/jova-core-primitives/`, `jova-core-chains/`, `jova-core/`, `jova-core-ffi/`, `jova-core-wasm/` — each builds, each has a `lib.rs` with one trivial `pub fn`.
- `bindings/swift/`, `bindings/kotlin/`, `bindings/wasm/` — each builds against the trivial Rust API.
- `spec/test-vectors.json` v0 with one trivial mnemonic-validation vector.
- `spec/test-vectors.schema.json`.
- `.github/workflows/`:
  - `ci.yml` — Rust tests on Linux/macOS/Windows.
  - `ci-bindings-swift.yml` — XCFramework build + Swift hello-world test.
  - `ci-bindings-kotlin.yml` — AAR build + JUnit hello-world.
  - `ci-bindings-wasm.yml` — **compile smoke test from day one**, plus vitest hello-world. WASM functional vector tests come in Phase 6, but the *compile* must stay green continuously.
  - `ci-no-std.yml` — `jova-core-primitives` builds for `thumbv7em-none-eabihf`.
  - `audit.yml` — `cargo-audit` + `cargo-deny` + `cargo-vet`.
- `deny.toml` enforcing license whitelist, advisory denylist, layered-dependency rule.
- `tools/verify-spec/` — fails CI if `docs/api.md` and `spec/api.md` disagree.

### Trivial test on each binding

Each binding's test loads the trivial vector and asserts `JovaWallet.isValidMnemonic("invalid words", "")` returns `false`. Just enough to prove the toolchain works end-to-end.

### WASM gating

If a chain crate identified in Phase -1 cannot compile to WASM, the WASM CI job builds with that chain feature-flagged off and the `bindings/wasm/` documentation flags the gap honestly.

### Tag

`v0.0.1`

---

## Phase 1 — Ethereum end-to-end on native bindings (10–14 days)

**Goal:** The contract becomes real. Mnemonic generation, address derivation, EIP-1559 signing, EIP-191, EIP-712 — fully implemented in `jova-core-chains::evm`, surfaced through `jova-core`, exercised on **Rust + Swift + Kotlin** with vector parity. WASM compiles but its functional tests come in Phase 6.

### Why 10–14 days, not 7–10

The previous estimate underweighted the FFI surface. EIP-712 typed-data, error mapping, memory-test wiring across three bindings, and the first round of vector authoring add real days. Honest range.

### Deliverables

- `crates/jova-core-primitives/src/`: complete `Mnemonic`, `Seed`, `XPrv`, `XPub`, `DerivationPath`, BIP-32 derivation, secp256k1 signing primitive. `Zeroize` everywhere. `no_std` build green.
- `crates/jova-core-chains/src/evm/`: `EvmSigner` impl `ChainSigner`. EIP-55 address derivation, EIP-1559 (type-2) tx signing using `alloy`. Access-list support. EIP-191 personal_sign. EIP-712 v4 typed-data signing.
- `crates/jova-core/src/`: complete public API surface for the EVM family.
- `crates/jova-core-ffi/`: every public method exported, error mapping done.
- `crates/jova-core-wasm/`: same. **Compiles only.** Functional tests in Phase 6.
- `bindings/swift/Sources/JovaCore/Convenience.swift`: written.
- `bindings/kotlin/jova-core/src/main/kotlin/io/jova/core/Convenience.kt`: written.
- `bindings/wasm/src/index.ts`: written. Smoke import-and-call test only.
- `spec/test-vectors.json` v1: 18+ vectors covering ETH (3 address × 2 mnemonics + 4 sign_tx scenarios + 2 sign_message + 3 error). EIP-1559 reference vectors imported from EIPs documentation.
- Property tests for EVM in `crates/jova-core/tests/properties/evm.rs`.
- Fuzz targets for EIP-1559 RLP, EIP-712 typed-data, and address parsing.
- All vectors pass on Rust, Swift, Kotlin. WASM passes the compile + smoke test.

### Exit criteria

- A signature produced by Rust, Swift, and Kotlin for every vector is byte-identical.
- The WASM crate builds and a JS hello-world can construct a wallet and read an address (no functional vector parity required yet).
- `cargo test --workspace --release --locked` is green.
- `cargo miri test -p jova-core-primitives` is green.
- All bindings' surface tests confirm every documented method exists.

### Tag

`v0.1.0`

This is where the contract is stress-tested. Any disagreement between bindings must be resolved here, before the surface grows.

---

## Phase 2 — Bitcoin (3–4 weeks)

**Goal:** BIP-84 native SegWit, PSBT signing, BIP-322 messages — fully implemented and parity-tested across native bindings.

### Why BTC second, not last

- **Funds-on-chain risk is highest.** Existing Android users hold real BTC at BIP-84 addresses; a derivation or signing bug means lost funds.
- **Migration risk is highest.** The Android app already derives `bc1q…`; we must produce the same addresses from the same seeds, byte-identical.
- **Semantic surface is largest.** PSBT v1, multi-input, multi-party, BIP-322 + legacy `signMessage`, address validation. The earlier we surface unexpected complexity, the more time we have.

Doing the riskiest chain first while attention is fresh and the team is unfatigued is the better trade.

### Deliverables

- `crates/jova-core-chains/src/btc/` using `bdk_wallet`. BIP-84 derivation (P2WPKH `bc1q…`).
- PSBT signing for single-input, multi-input, and multi-party scenarios.
- BIP-322 message signing with legacy `signMessage` fallback.
- Address derivation, address validation.
- Vectors: BIP-84 official vectors + manually constructed multi-input PSBTs + BIP-322 reference + legacy fallback.
- Reconcile against the legacy Android app's existing storage by spot-checking 100 known seed → `bc1q…` mappings.
- Property tests in `tests/properties/btc.rs`.
- Fuzz targets for PSBT decode, address validation, BIP-322 verification.
- All vectors green on Rust + Swift + Kotlin.

### Exit criteria

- BIP-84 official test vectors pass byte-identically on every native binding.
- A signing round-trip on a real testnet PSBT (manual smoke test) produces a valid signed transaction.
- Spot-check pass: 100 legacy-Android-derived addresses match SDK derivation.

### Tag

`v0.2.0`

---

## Phase 3 — Solana + XRP + remaining EVM chains (3–5 weeks)

**Goal:** Every remaining v1 chain shipping with full vector parity.

### Why bundled

These three chunks are mostly independent and can run in parallel. None individually carries BTC's risk profile.

### 3a — Other EVM chains (~2 days)

Polygon, BSC, Arbitrum, Optimism, Base. Same signing path; only `chainId` changes. Mostly enum entries + vector additions. `customEvm(N)` variant. Vectors mostly captured from local `anvil` instances configured per chainId.

### 3b — XRP (~5–7 days)

`crates/jova-core-chains/src/xrp/` using `xrpl-rust`. Address derivation (base58check `r…`), canonical XRPL serialization, secp256k1 signing.

XRPL canonical serialization has surprising ordering rules (field ordering by type code + sort order). Differential test against `xrpl-py` for at least 20 random tx shapes. Vectors from XRPL test cases + known-good local `rippled` signing.

### 3c — Solana (~7–10 days)

`crates/jova-core-chains/src/sol/` using Anza's Solana split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`).

SLIP-10 derivation (ed25519). VersionedTransaction (v0) signing with ALT support. Raw ed25519 message signing.

Vectors from real mainnet txs with known seeds, plus property-based round-trip tests. ALT-using messages get their own vector category.

### Exit criteria

- All vectors green on Rust + Swift + Kotlin for every chain.
- Differential XRPL test against `xrpl-py` passes on 100+ random shapes.
- Solana versioned-tx tests cover both legacy-message-in-v0 and ALT-using cases.

### Tag

`v0.5.0`

---

## Phase 4 — iOS and Android app integration with feature flags (3–4 weeks)

**Goal:** Both apps signing through `jovawallet-core` end-to-end on mainnet. Old crypto stacks deleted.

### Feature-flag the migration

Cut over per chain, not per app. The pattern:

```
WalletService.signEthereum(...) {
    if FeatureFlag.useJovaCoreForEthereum {
        return jovaCore.sign(...)
    } else {
        return legacyTwc.sign(...)
    }
}
```

Per-chain flags, server-controlled, per-user-cohort rollout: 1% → 10% → 50% → 100%. A bug found at 1% is a kill switch away from a rollback. After 100% on mainnet for two weeks, delete the legacy code path.

### 4a — iOS integration (~1.5–2 weeks)

- iOS app pulls `JovaCore` via SwiftPM, pinned to `0.5.0`.
- `WalletService` (per `integration-ios.md`) implemented in the app.
- Per-chain feature flag plumbing.
- Migrate `CryptoWalletService.swift` call sites with flag gate.
- BTC and SOL signing paths wired up (they were stubs before).
- WalletConnect bridge updated to call `WalletService` (flag-gated).
- Manual end-to-end mainnet smoke test per chain at 1% rollout. Small amounts.
- Promote to 100% after two weeks at 50%.
- Delete legacy `CryptoWalletService.swift` and `TrustWalletCore` SwiftPM dependency.

### 4b — Android integration (~2 weeks)

- Android app pulls `io.jovachain:jova-core` from Maven Central, pinned to `0.5.0`.
- `WalletRepository` (per `integration-android.md`) implemented.
- BTC address reconciliation: spot-check 100 known addresses against legacy storage.
- Migrate `EvmSigner`, `SecureWalletDerivation`, `BitcoinWalletManager` call sites with flag gate.
- WalletConnect (Reown) bridge updated.
- Same staged rollout: 1% → 10% → 50% → 100% per chain.
- Delete legacy code; remove `web3j`, `bitcoinj`, `bdk-android` from `build.gradle.kts`.
- Verify APK shrinks ~15 MB.

### Exit criteria

- Both apps in production with `jovawallet-core` as their only signing dependency at 100% rollout for at least one full release cycle.
- App-side telemetry shows `JovaError` rates within expected envelope (no spike in `internal` errors, no chain showing elevated `signingFailed` rates).
- Legacy code paths deleted.

(No new SDK tag for this phase; the SDK doesn't change, only its usage.)

---

## Phase 5 — Hardening, audit, RC, v1.0 (3–4 weeks)

**Goal:** v1.0 release. The API contract is locked from this point.

### Deliverables

- **External audit.** Paid review of `jova-core-primitives`, `jova-core-chains`, `jova-core`, and the FFI handle lifecycle. Findings addressed.
- **Fuzz hardening.** 14 days of nightly fuzz with no new crashes across all targets.
- **Memory test pass.** `cargo miri test` clean. Every binding's `MemoryTests` confirms zeroization.
- **Property test depth.** 4096 cases per property in CI; nothing flakes.
- **Threat-model document.** `security.md` cross-checked against actual implementation; gaps closed.
- **Reproducible-build verification.** Two engineers build from the tag locally and compare SHA-256 of every artifact.
- **CHANGELOG.md** lists every notable change v0.0.1 → v1.0.0.
- **`spec/api.md`** frozen as the v1.0 reference. Future minor versions append; never overwrite.
- **`spec/CHANGELOG.md`** opened.
- **Bug bounty program** drafted and funded; opens with v1.0 announcement.

### Release-candidate cycle

- `v1.0.0-rc.1` tag → full release pipeline runs in **dry-run mode**: artifacts built, staged, and verified in test-flight environments without final publish.
- Manual smoke-test consumption from staged Maven (OSSRH staging repo, not production), staged npm (under a `-rc` dist-tag), and a private branch of the satellite Swift repo.
- If RC.1 passes, tag `v1.0.0`. If issues found, fix and tag `v1.0.0-rc.2`. Iterate.

See `build-and-release.md` for the staged publish flow.

### Tag

`v1.0.0` — only after at least one clean RC cycle.

---

## Phase 6 — WASM functional + npm publish (2–3 weeks)

**Goal:** `@jovachain/wallet-core` published to npm with full functional vector parity.

### Why this is its own phase

WASM has been compile-smoking since Phase 0, but functional vector parity, bundle-size optimization, Web Worker integration patterns, and tree-shakeable per-chain entrypoints all need real work. None of it should block the v1.0 native release; all of it ships when ready.

If Phase -1 identified a chain crate that fights WASM, **that chain may ship to WASM later than the others**. The npm package documents which chains are functional vs. native-only.

### Deliverables

- `crates/jova-core-wasm/` polish: bundle size optimization, per-chain entrypoints, TypeScript types refined.
- `bindings/wasm/`: ESM + CJS package, Web Worker example, `Symbol.dispose` support.
- `examples/web-sample/`: production-quality Vite app demonstrating the worker pattern.
- `examples/backend-node/`: Express server demonstrating signing in Node with mnemonic loaded from a Vault dev instance.
- Documentation updates: `integration-web.md` finalized, `integration-backend.md` finalized.
- `@jovachain/wallet-core` published to npm at `1.x`.

### Tag

`v1.1.0` (additive; no breaking change).

---

## Phase 7 — Hardware-wallet readiness (4–6 weeks)

**Goal:** A reference Cortex-M firmware integrating `jova-core-primitives` exists and signs against the same vectors.

### Deliverables

- `external-rng` feature on `jova-core-primitives`.
- `from_seed_bytes(bytes: &[u8])` constructor on `JovaWallet` (direct Rust API only; not FFI).
- `examples/firmware-template/`: `thumbv7em-none-eabihf` Cortex-M firmware that uses `jova-core-primitives` to derive an EVM key and sign a digest. Runs on a STM32 dev board.
- Documentation in `integration-hardware.md` updated with production reference patterns.
- Side-channel and glitch-protection guidance, including secure-element integration patterns for ATECC and OPTIGA.

### Tag

`v1.2.0`.

---

## Phase 8 — Custom Jova chain (when it ships)

**Goal:** Native support for the Jova chain when it goes live.

### If the Jova chain is EVM-equivalent

- Use `JovaChain.customEvm(chainId: <jova-chain-id>)`.
- No code change.
- Add one vector triplet to `spec/test-vectors.json`.
- Tag a minor version.

### If non-EVM

- Add `JovaChain.jova` enum variant.
- Add `UnsignedTx.jova(...)` variant.
- New `chains::jova/` module implementing `ChainSigner`.
- Three vector triplets.
- Apps see no change until they want to use the new variant.
- Tag a minor version.

---

## Ongoing — beyond v1.0

These run continuously after v1.0 ships:

- **Dependency updates.** `cargo audit` daily; bumps within minor versions if no breaking change. Major-version bumps of underlying crates (e.g., `bdk_wallet 1.x → 2.x`) follow our own major version, so app teams can plan.
- **Vector additions.** New vectors append; existing ones never change.
- **Chain additions.** Per `chains.md`'s checklist. Each is a minor version bump.
- **Audit cadence.** Every 12 months, or whenever the underlying chain crates have a major-version change.
- **Fuzz corpus growth.** The `jovachain/jovawallet-core-fuzz-corpus` repo grows over time.

---

## Total realistic timeline

Adding the honest ranges:

| Phase | Duration |
|---|---|
| -1 | 3–5 days |
| 0 | 3–5 days |
| 1 | 10–14 days |
| 2 | 3–4 weeks |
| 3 | 3–5 weeks |
| 4 | 3–4 weeks |
| 5 | 3–4 weeks |
| **Sub-total to v1.0** | **~16–22 weeks** (4–5.5 months) |
| 6 | 2–3 weeks |
| 7 | 4–6 weeks |

A "12-week project" estimate is fantasy for this scope. Plan for 4–5 months to v1.0 with a senior team; 6+ months with a smaller or less-experienced one. The phases that *can* run in parallel (3a/3b/3c, 4a/4b) compress the wall-clock if you have the headcount.

---

## Success criteria

- Every binding published, every binding green on the same vector suite.
- Both apps shipping with `jovawallet-core` as their only signing dependency, at 100% rollout, with legacy code deleted.
- `git log` is the answer to "every change to crypto behavior, ever."
- A new chain takes hours to a few days, not weeks.
- A bug fix lands once and propagates to every binding at the next tag.
- Hardware wallet firmware reuses primitives unchanged. (Phase 7)
- External audit complete; findings remediated; v1.0.0 tagged. (Phase 5)
- Drift between bindings detected by CI before any disagreement reaches main.
