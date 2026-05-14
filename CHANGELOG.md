# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
