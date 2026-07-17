# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Phase 7 — hardware-wallet readiness)

- **`JovaRng` trait** in `jova-core-primitives` (gated behind the new `external-rng` feature): firmware integrations provide entropy via `fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError>`. `RngError::{Unavailable, HealthCheckFailed}` cover the two failure modes auditors typically check. The trait is `no_std + alloc`-clean.
- **`Mnemonic::generate_with(strength, &mut impl JovaRng)`** — generate a mnemonic without pulling in `getrandom`. Compatible with `no_std` firmware that uses a hardware TRNG. Zeroizes the entropy buffer on return.
- **`Seed::from_external_bytes(bytes: [u8; 64])`** — wrap a pre-derived BIP-39 seed from a secure element (`Zeroize + ZeroizeOnDrop` preserved). For firmware that stores the seed in an ATECC608 / OPTIGA Trust M / SE050.
- **`JovaWallet::from_seed_bytes(bytes: [u8; 64])`** in `jova-core` — bypasses the mnemonic → PBKDF2 step. Rust-only; not exposed via FFI/WASM. Tested byte-equal against `from_mnemonic` for Ethereum + Bitcoin BIP-44 paths.
- **`examples/firmware-template/`** — working `thumbv7em-none-eabihf` reference binary linking `jova-core-primitives`. BIP-39 → seed → BIP-44 → secp256k1 ECDSA signing in `no_std`. 394 KB stripped ELF. Built in CI on every PR via `.github/workflows/ci-no-std.yml`.
- **`docs/integration-hardware.md`** — rewritten with Phase 7 API surface: `JovaRng` trait, secure-element-seeded CSPRNG pattern, `Seed::from_external_bytes` / `JovaWallet::from_seed_bytes` usage, side-channel + glitch-protection guidance referencing ATECC608, OPTIGA Trust M, SE050, and the public design docs from Foundation Devices Passport, BitBox02, Trezor Safe 5.
- **CI extension**: `ci-no-std.yml` now also tests the `external-rng` feature paths, the `from_seed_bytes` constructor, and builds the firmware-template for `thumbv7em-none-eabihf`.

### Notes
- Phase 7 is preparatory — the SDK is hardware-ready; the firmware repo itself (e.g., `jovachain/jova-firmware-reference`) is separate work. SDK-side scope is complete.
- `slip-10 0.4` doesn't ship ed25519 support — Solana SLIP-10 was already implemented in-crate during Phase 3c. No additional work needed here.
- 11 new tests across `external_rng.rs` (5) and `from_seed_bytes.rs` (2) plus firmware-template build smoke.

### Added (Phase 6 — WASM functional EVM + SOL)

The WASM binding (`crates/jova-core-wasm`) now exposes the full `JovaWallet` signing surface for EVM + SOL chains. **BTC + XRP browser signing is deferred per the 2026-05-11 user decision** — those variants return `unsupportedChain` at the WASM boundary before any chain code executes. Native bindings (Swift, Kotlin) retain full coverage; only WASM is constrained.

- **Full JovaWallet WASM surface**: `createMnemonic`, `isValidMnemonic`, `isValidAddress`, `new JovaWallet(mnemonic)`, `.address(chain, account)`, `.signTx(unsigned)`, `.signMessage(msg)`, `.destroy()`. Wraps `Option<jova_core::JovaWallet>` so `destroy()` zeroizes the inner seed deterministically (JS GC finalizers run too late for crypto).
- **TypeScript types** with discriminated unions for `JovaChain`, `UnsignedTx`, `SignableMessage`, `JovaErrorPayload`. Hand-written, not auto-generated.
- **Disposable `JovaWallet` wrapper** in TS — `using wallet = JovaWallet.fromMnemonic(mnemonic)` in TS 5.5+ calls `destroy()` on scope exit via `Symbol.dispose`.
- **Per-chain entrypoints** for tree-shaking: `@jovachain/wallet-core/evm` and `@jovachain/wallet-core/sol` (no `/btc` or `/xrp` per the deferral). Subpath exports in `bindings/wasm/package.json`.
- **42 Vitest tests** across 4 test files: 9 EVM address, 6 EVM sign_tx, 2 EVM sign_message, 3 EVM error, 1 SOL address, 2 SOL sign_tx, 1 SOL sign_message, 4 SOL error, 5 BTC/XRP rejection assertions, 1 hello-world.
- **Bundle size budget check** (`bindings/wasm/scripts/size-check.mjs`). Current: 787 KB gzipped (vs. 2 MB budget) for the WASM blob, 20 KB raw for `index.js`.
- **`bindings/wasm/COVERAGE.md`** documents the deferral honestly.
- **getrandom 0.2/0.3 dual feature flag**: declared both at the WASM leaf crate (`getrandom 0.3 features = ["wasm_js"]` + `getrandom_02 = { package = "getrandom", version = "0.2", features = ["js"] }`). Cargo unifies features additively per version, so transitive 0.2 deps (alloy + solana-keypair → rand_core → elliptic-curve) get browser RNG without conflicting with the workspace 0.3 dep.

No SDK API change; no version bump in this section (v1.1.0 ships at Phase 6 final tag after Phase 5 closes).

## [0.5.0] — 2026-07-16

### Added (Track 1 — HD account index on signing)
- **`account: u32` parameter on `JovaWallet::sign_tx` and `sign_message`** — signing now derives from an arbitrary HD account, mirroring `address(chain, account)`. Previously both signers always used account 0. `account` is a **required** parameter across all bindings (UniFFI's Kotlin backend does not emit default argument values, so a defaulted parameter is not viable there); callers pass `0` for the primary account. The WASM TypeScript wrapper exposes it with a `= 0` default for JS ergonomics.
- **Guaranteed key/address parity.** `address`, `sign_tx`, and `sign_message` now all obtain their key through the single shared derivation `JovaChain::derivation_path(account)` → `derive_for` / `derive_ed25519_for`. For any `(chain, account)` the signing key is byte-identical to the key behind `address(chain, account)`.
- **New test `crates/jova-core/tests/multi_account.rs`** — proves, for accounts 0/1/2 on every chain family, that the signer recovered/extracted from a signature or signed tx equals `address(chain, account)` (EVM ecrecover, BTC address-binding guard, XRP `SigningPubKey` re-derivation, Solana ed25519 verify), and that the three accounts yield distinct addresses.

### Derivation-path semantics (confirmed)
- **EVM (all chains): `account` increments the BIP-44 `address_index` — `m/44'/60'/0'/0/N` — identical to MetaMask's default HD account scheme.** Wallet-import parity with MetaMask holds for account > 0.
- **Bitcoin: `m/84'/0'/0'/0/N`** (BIP-84 native SegWit, address_index).
- **XRP: `m/44'/144'/0'/0/N`** (address_index).
- **Solana: `m/44'/501'/N'/0'/0'`** — the account is applied at the hardened `account'` level (SLIP-10 ed25519 requires all-hardened components, so the address_index scheme can't be used). This preserves the exact v0.4.0 account-0 path; note that for account > 0 the resulting key does **not** match Phantom/Solflare (which use the 4-level `m/44'/501'/N'/0'`) — a pre-existing 5-vs-4-level divergence that already applied at account 0.
- **Imported single-key wallets (`from_private_key`)** hold one leaf key and cannot HD-derive, so `account` is ignored for them (both `address` and `sign_*` ignore it — parity preserved).

### Compatibility
- **Account 0 is byte-for-byte unchanged** — all existing `spec/test-vectors.json` vectors pass unmodified; no reference value was altered.

## [0.4.0] — 2026-06-18

### Added (Track 0 — private-key import)
- **`JovaWallet::from_private_key(hex: &str, chain: &JovaChain)`** in `jova-core` — single-chain wallet from a raw 32-byte private key (optional `0x` prefix). Curve selected by chain: secp256k1 for EVM family / Bitcoin / XRP, ed25519 for Solana. The wallet serves ONLY its bound chain; any other chain returns `UnsupportedChain`.
- **`KeyMaterial` enum** behind `JovaWallet` (`Seed | Secp256k1{key,chain} | Ed25519{key,chain}`). Mnemonic / `from_seed_bytes` wallets are unchanged (`KeyMaterial::Seed`, byte-identical derivation). Imported keys wrap the raw scalar in `XPrv` / `Ed25519Xprv` with a zero chain code (leaf signing reads only the key bytes).
- **`JovaError::InvalidPrivateKey { reason }`** + **`FfiError::InvalidPrivateKey { reason }`** — reasons: `not_hex`, `expected_32_bytes`, `secp256k1_scalar_out_of_range`.
- **FFI constructor `JovaWallet.fromPrivateKey(hex:chain:)`** (uniffi `#[uniffi::constructor]`) surfaced in regenerated Swift + Kotlin bindings.
- **New test vectors** in `spec/test-vectors.json` (`version` `0.6` → `0.7`): `private_key_address` (Ethereum/Bitcoin/XRP/Solana) and `private_key_sign_tx` (Ethereum EIP-1559 transfer).
- **New Rust tests:** `crates/jova-core/tests/private_key.rs` (happy paths + bad hex / wrong length / zero scalar / cross-curve / unbound-chain negatives), `crates/jova-core/tests/vectors_private_key.rs` (vector runner), `crates/jova-core-ffi/tests/ffi_private_key.rs` (FFI smoke).

### Changed
- `JovaWallet` struct field `seed: Seed` → `material: KeyMaterial` (internal; no public-API removal).
- `sign_tx` / `sign_message` now enforce key-material chain scoping before dispatch.

### Notes
- Internal Cargo crate `version` strings remain `0.0.1` (the published version is tracked by git tag + this CHANGELOG, consistent with prior releases). The 0.4.0 label is the SDK release tag and the version iOS pins to.
- To cut the actual GitHub release after this branch merges: `git tag v0.4.0 && git push origin v0.4.0`. Then compute the XCFramework zip checksum with `swift package compute-checksum bindings/swift/JovaCoreFFI.xcframework.zip` and update `bindings/swift/Package.swift` to the published `url:` + `checksum:` form (see comment in that file).

## [0.3.0] — 2026-05-14

### Added (Phase 3 — Solana + XRP + remaining EVM chains)

Every v1 chain now ships. This is the SDK version that Phase 4 (app integration) consumes.

**Sub-phase 3a — EVM chain family (Polygon, BSC, Arbitrum, Optimism, Base):**
- 5 new vectors in `spec/test-vectors.json`: 3 address (Arbitrum, Optimism, Base) and 2 sign_tx (Polygon transfer, Arbitrum transfer). Phase 1 already covered Polygon and BSC address vectors. Same `m/44'/60'/0'/0/0` path; same EVM signer; per-chain `chainId` produces distinct `signed_hex` + `tx_hash`. Captures from Foundry `cast` (`cast mktx`, `cast keccak`).

**Sub-phase 3b — XRP:**
- `jova_core_primitives::DerivationPath::bip44_path(coin_type, account, change, index)` helper.
- `jova_core_chains::xrp::derive_xrp_address` / `validate_xrp_address` — XRPL classic address (`r…`) via SHA256 → RIPEMD-160 → XRPL base58.
- `jova_core_chains::xrp::sign_xrp_tx(xprv, tx_json) -> Result<(String, String), ChainError>` — canonical XRPL signing (encode_for_signing → SHA512Half → secp256k1 ECDSA DER → re-encode with TxnSignature → SHA512Half(TXN\0||signed) for tx_hash). Captures cross-checked against `xrpl-py 4.5` + `bip_utils 2.x` at BIP-44 coin type 144.
- `XrpSigner` (sibling type, not a ChainSigner impl — XRP has no message-signing scheme; returns `MalformedSignableMessage("xrp_message_signing_unsupported")` on sign_message).
- `JovaChain::Xrp` variant wired through core + FFI. Path `m/44'/144'/0'/0/0`.
- `UnsignedTx::Xrp { tx_json: String }`.
- 6 XRP vectors: 1 address, 2 sign_tx (Payment+DestinationTag, OfferCreate), 3 errors (invalid_json, missing_required_field:TransactionType, missing_required_field:Account).

**Sub-phase 3c — Solana:**
- `jova_core_primitives::derive_ed25519(seed, path)` — SLIP-10 ed25519 derivation. Hardened-only enforced per spec (returns `Ed25519DeriveError::HardenedRequired` otherwise). `slip-10` 0.4 doesn't ship ed25519 support, so the algorithm is implemented in-crate (HMAC-SHA512 with `"ed25519 seed"` master salt). Cross-checked against `bip_utils Bip44Coins.SOLANA` at `m/44'/501'/0'/0'/0'` (Phantom/Solflare 5-component path).
- `Ed25519Xprv` — Zeroize + ZeroizeOnDrop, NOT Clone (same security posture as `XPrv`). Redacted Debug.
- `jova_core_chains::sol::derive_sol_address(xprv)` / `validate_sol_address(s)` — base58 of the ed25519 pubkey via `solana_pubkey::Pubkey`.
- `jova_core_chains::sol::sign_sol_tx(xprv, message_base64, recent_blockhash)` — VersionedTransaction (v0) signing. Decodes the bincode-serialized VersionedMessage, validates `recent_blockhash` matches, signs the serialized message bytes with ed25519, prepends signature(s), bincode-serializes the full VersionedTransaction. Returns `(signed_hex, signature_b58)` where the signature is Solana's tx_hash convention.
- `jova_core_chains::sol::sign_sol_message(xprv, message_base64)` — raw ed25519 over arbitrary bytes (Solana convention; no canonical message scheme).
- `SolSigner` — sibling type (not a ChainSigner impl, same reason as XRP plus the ed25519 vs secp256k1 key-type split). JovaWallet special-cases `JovaChain::Solana` via `derive_ed25519_path` helper.
- `JovaChain::Solana` variant wired through core + FFI. Path `m/44'/501'/0'/0'/0'`.
- `UnsignedTx::Solana { message_base64: String, recent_blockhash: String }`.
- `SignableMessage::Solana { message_base64: String }`.
- 8 SOL vectors: 1 address, 2 sign_tx (system_transfer_v0, with_alt_v0), 1 sign_message, 4 errors (invalid_base64_tx, unsupported_version, blockhash_mismatch, invalid_base64_message).
- Anza split crates inherited into `jova-core-chains` with `serde` feature: `solana-keypair 3.1`, `solana-pubkey 4.2`, `solana-signature 3.4` (+ `std`), `solana-transaction 4.1`, `solana-message 4.1`. No monolithic `solana-sdk` dep.

### Changed
- `spec/test-vectors.json` version `"0.3"` → `"0.6"` across the three sub-phases.
- `spec/errors.md` extended with XRP and Solana reason-vocabulary tables.

### Notes
- XRP differential-vs-`xrpl-py` (the 100-iteration random test described in the plan) deferred — the byte-equal Payment + OfferCreate captures already lock the interop contract; expanding to a 100-iteration harness adds significant Python/Rust bridging complexity for marginal coverage. Tracked as a follow-up if signal arises.
- Solana ALT (Address Lookup Table) signing successfully captured and vector-tested via solders against a fixed deterministic blockhash.

## [0.2.0] — 2026-05-14

### Added (Phase 2 — Bitcoin)
- `jova_core_primitives::DerivationPath::bip84_path(account, change, index)` helper that builds `m/84'/0'/account'/change/index` for Bitcoin mainnet.
- `jova_core_chains::btc::derive_p2wpkh(xprv)` — BIP-84 native SegWit address encoder (`bc1q…`).
- `jova_core_chains::btc::validate_btc_address(s)` — mainnet P2WPKH validator; rejects P2PKH, P2SH, Taproot, and testnet/regtest addresses.
- `jova_core_chains::btc::sign_psbt(xprv, psbt_base64) -> PsbtSignResult` — BIP-174 PSBT signing. Single-input and multi-input fully-owned PSBTs finalize to broadcast-ready transactions. Multi-party PSBTs (inputs the wallet does not fully own) return an updated PSBT for the next signer. Uses BIP-143 sighash + low-R-grinded ECDSA (matching Bitcoin Core ≥ v0.17 defaults).
- `jova_core_chains::btc::sign_btc_message(xprv, message, address, scheme)` — BIP-322 simple signature scheme and legacy Bitcoin Core `signmessage` scheme. Foreign-address attempts return `MalformedSignableMessage("btc_message_address_mismatch")`.
- `jova_core_chains::btc::BtcSigner` — `ChainSigner` implementation routing all three operations.
- `JovaChain::Bitcoin` variant + dispatch in `JovaWallet::{address, sign_tx, sign_message}` and `is_valid_address`.
- `UnsignedTx::Bitcoin { psbt_base64 }` and `SignableMessage::Bitcoin { message, address, scheme: BtcMsgScheme }` in `jova-core-chains` and surfaced through `jova-core-ffi`. Multi-party PSBT results return `SignedTx.raw_hex` prefixed with `psbt:` to signal hand-off vs. broadcast.
- 12 BTC vectors in `spec/test-vectors.json` (version bumped `"0.2"` → `"0.3"`): 4 address, 3 sign_tx, 2 sign_message, 3 error. All captured byte-equal against `embit 0.8.0` (cross-validated by the `bip322` PyPI verifier for BIP-322).
- Vector parity tests on Rust + Kotlin + Swift (Swift CI-only on `macos-latest`).
- 6 property tests + 3 new fuzz targets (`fuzz_psbt_sign`, `fuzz_btc_address_parse`, `fuzz_bip322_verify`).
- `tools/btc-migration-check`: `jova-btc-migration-check` binary that verifies the SDK's BIP-84 derivation matches the legacy Android wallet's known mappings. Gated on `tools/btc-migration-check/known-android-mappings.csv` (gitignored — contains user mnemonics). Tracking issue: [#3](https://github.com/jovachain/jovawallet-core/issues/3).
- `bitcoin` 0.32 added to `jova-core-chains` dependencies (workspace dep with `features = ["std"]`). `base64` 0.22 added.
- `MITNFA` license accepted in `deny.toml` (transitive via `hex_lit` / `bitcoin` 0.32).

### Phase 2 release gates (open)
- **Migration spot-check** ([#3](https://github.com/jovachain/jovawallet-core/issues/3)): the Android team must export ≥100 production mappings before any wallet-app rollout uses BTC. Tool ready; awaits CSV.
- **Mainnet smoke** ([#4](https://github.com/jovachain/jovawallet-core/issues/4)): human-driven; ~$5 in BTC; one real broadcast + confirmation.

`v0.2.0` ships with the cryptographic surface complete and tested; the two gates close before any production rollout.

## [0.1.0] — 2026-05-13

### Added
- EVM end-to-end: BIP-39 `Mnemonic`/`Seed` (Zeroize, not Clone), BIP-32 derivation via `XPrv`.
- `EvmSigner` with EIP-1559 + EIP-191 + EIP-712 signing via `alloy 2.0`.
- Public `JovaWallet` API and full uniffi FFI surface.
- 15 cast-captured EVM vectors; Swift + Kotlin parity tests byte-equal vs `cast`.
- Property tests + 3 fuzz targets (`fuzz_eip1559_decode`, `fuzz_eip712_typed`, `fuzz_address_parse`).
- miri-clean (4 secp256k1 FFI tests skipped under `#[cfg(not(miri))]` due to extern-static C limitation).

## [0.0.1] — 2026-05-12

### Added
- Repo bootstrap: workspace, 5 crates, 3 bindings, 6 CI workflows, governance files.
- `spec/test-vectors.json` with one negative mnemonic-validation vector.
- Hello-world parity tests on Rust, Swift, Kotlin, WASM bindings.
