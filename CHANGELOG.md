# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
