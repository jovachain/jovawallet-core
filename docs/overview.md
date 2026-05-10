# Overview

## What `jovawallet-core` is

A single signing SDK for the Jova wallet ecosystem, implemented as a Rust core and shipped as native packages to every platform Jova runs on.

It is **the only place signing happens** in the Jova product surface. The iOS app, the Android app, the future web wallet, the future backend, and the future hardware wallet all call into the same Rust source tree. A bug fix is fixed once. A new chain is added once. Drift in the *crypto layer* is prevented by having one implementation; drift in FFI marshalling, enum mapping, and binding-side conveniences is detected by shared test vectors at the binding boundary and prevented from merging.

## Why it exists

Today the Jova iOS and Android apps each carry their own crypto stack. iOS uses Trust Wallet Core directly. Android uses a mix of `web3j`, `bitcoinj`, and `bdk-android`. The two apps disagree on:

- Bitcoin derivation path (iOS legacy stub uses BIP-44; Android uses BIP-84).
- Solana signing (iOS stubbed; Android partially wired through `solana-mobile`).
- EVM message signing (different EIP-712 implementations on each side).

This is exactly the failure mode `jovawallet-core` is designed to eliminate. **One contract, one core, one set of test vectors that every binding must pass.**

## What it does

- **Mnemonics.** Generate (12 or 24 words), validate, derive seed.
- **Addresses.** Derive and validate addresses on every supported chain, with deterministic results across every platform.
- **Transaction signing.** EVM (EIP-1559), Bitcoin (BIP-174 PSBT, BIP-84 native SegWit), Solana (v0 versioned tx), XRP (canonical XRPL).
- **Message signing.** EIP-191 personal_sign, EIP-712 typed data v4, BIP-322 Bitcoin messages, raw ed25519 for Solana.

## What it does not do

These are app, backend, or future-module concerns and stay out by design:

| Concern | Where it lives |
|---|---|
| RPC, balance fetching, broadcast | Backend (`jova-rpc`, future) |
| Fee estimation, gas oracle, RBF | Backend |
| Secret storage at rest | App: iOS Keychain, Android Keystore |
| Biometrics, PIN, UI | App |
| WalletConnect protocol negotiation | App |
| Push notifications, webhooks | Backend |
| User account, multi-device sync | Backend |

If a feature would otherwise grow `jovawallet-core` past "signing primitives," it goes in a new module — never here. ADR D7 in `decisions.md` locks this.

## Who consumes it

| Consumer | Binding | Status |
|---|---|---|
| iOS app | SwiftPM package `JovaCore` | v1 launch target |
| Android app | Maven AAR `io.jovachain:jova-core` | v1 launch target |
| Web wallet | npm package `@jovachain/wallet-core` (WASM) | Phase 6 |
| Backend services | Rust crate `jova-core` (direct) or WASM | Phase 6 |
| Hardware wallet firmware | Rust crate `jova-core-primitives` (no_std) | Phase 7 |
| Internal CLI / tooling | Rust crate `jova-core` (direct) | Phase 0 onward |

Every binding is generated from the same Rust workspace. There is no language-specific reimplementation of crypto code anywhere in the Jova product.

## High-level architecture

```
                 ┌────────────────────────────────────────┐
                 │           Rust workspace               │
                 │                                        │
                 │   jova-core-primitives (no_std)        │
                 │           ▲                            │
                 │           │                            │
                 │   jova-core-chains (std)               │
                 │           ▲                            │
                 │           │                            │
                 │       jova-core (public Rust API)      │
                 │           ▲                            │
                 │   ┌───────┴────────┐                   │
                 │   │                │                   │
                 │ jova-core-ffi   jova-core-wasm         │
                 │  (uniffi-rs)   (wasm-bindgen)          │
                 └─────┬──────┬─────────┬─────────────────┘
                       │      │         │
        ┌──────────────┘      │         └──────────────┐
        ▼                     ▼                        ▼
   Swift package        Kotlin AAR                 npm package
   (iOS, macOS)         (Android, JVM)             (browser, Node)

   Hardware firmware imports jova-core-primitives directly.
   Backend Rust services import jova-core directly.
```

See `architecture.md` for the full drawing and the rationale behind each crate boundary.

## Contract guarantees

These are the things `jovawallet-core` promises to its consumers and never breaks:

1. **Determinism.** A given mnemonic + chain + tx input produces byte-identical output on every binding, every platform, forever. CI enforces this against `spec/test-vectors.json`.
2. **No type leak.** No file outside the FFI/WASM crates references the underlying Rust crypto crates. Apps see Swift types, Kotlin types, JS types — never `bdk_wallet::Wallet` or `alloy::primitives::Address`.
3. **No surprise allocations of secrets.** Mnemonic words, seeds, and private keys are wrapped in `zeroize::Zeroizing` types and cleared on drop. Bindings extend this guarantee where the host language permits.
4. **Lockstep versions.** Every published binding for a given Git tag is built from the same commit. There is no `swift-1.4.2` vs `kotlin-1.4.3`.
5. **Additive minor versions.** Adding a chain or a method is minor. Removing or changing one is major. Vector files are append-only between major versions.

## Status

Pre-implementation. Phase 0 of [`plan.md`](./plan.md) creates the workspace, lands a hello-world test on every binding, and tags `v0.0.1`.
