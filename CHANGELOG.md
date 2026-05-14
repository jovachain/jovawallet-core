# Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Phase 2 — in progress on `feat/phase-2-bitcoin`)
- `jova_core_primitives::DerivationPath::bip84_path(account, change, index)` helper that builds `m/84'/0'/account'/change/index` for Bitcoin mainnet.
- `jova_core_chains::btc::derive_p2wpkh(xprv)` — BIP-84 native SegWit address encoder (`bc1q…`).
- `jova_core_chains::btc::validate_btc_address(s)` — mainnet P2WPKH validator; rejects P2PKH, P2SH, Taproot, and testnet/regtest addresses.
- `bitcoin` 0.32 added to `jova-core-chains` dependencies (workspace dep with `features = ["std"]`).
- BIP-84 official test vectors covered: 3 compressed pubkeys + 3 first-receive addresses derived byte-identically.

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
