# jovawallet-core

The single signing SDK for the Jova wallet ecosystem. A pure-Rust core wrapped with `uniffi-rs` and `wasm-bindgen` so iOS, Android, web, backend, and (eventually) hardware wallets all sign through the same audited code.

> **Status:** design complete, implementation pending Phase 0 bootstrap. See [`docs/`](./docs/README.md).

## What it does

- Generate / import / validate BIP-39 mnemonics
- Derive addresses (BTC, SOL, EVM family, XRP, custom Jova chain)
- Sign transactions and messages on every supported chain
- Run identically on every Jova platform — drift is detected by shared vectors at the binding boundary and prevented from merging

## What it doesn't do

No RPC, no secret storage, no fee estimation, no WalletConnect protocol, no UI. Those live in the apps and the backend. Keeping `jovawallet-core` strictly limited to signing primitives is what makes it auditable, testable against vectors, and portable to every platform.

## Layout (target)

```
jovawallet-core/
├── crates/                  ← Rust workspace; the single source of crypto truth
│   ├── jova-core-primitives    no_std-clean; lands on hardware wallets
│   ├── jova-core-chains        per-chain encoding (BDK, alloy, Anza Solana split crates, xrpl-rust)
│   ├── jova-core               public Rust API
│   ├── jova-core-ffi           uniffi-rs → Swift + Kotlin
│   └── jova-core-wasm          wasm-bindgen → JavaScript / WASM
├── bindings/                ← language-native consumer packages
│   ├── swift/                  SwiftPM (XCFramework)
│   ├── kotlin/                 Maven AAR
│   └── wasm/                   npm
├── spec/                    ← correctness oracle every binding must pass
│   ├── api.md
│   ├── chains.md
│   └── test-vectors.json
├── docs/                    ← internal documentation
└── examples/                ← sample apps per binding
```

## Engine

Pure-Rust crates per chain. We do **not** wrap Trust Wallet Core.

| Chain | Crate |
|---|---|
| Bitcoin | `bdk_wallet`, `rust-bitcoin` |
| Ethereum + EVM family | `alloy` |
| Solana | Anza split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`) |
| XRP | `xrpl-rust` |
| Primitives | `secp256k1`, `ed25519-dalek`, `bip39`, `slip-10`, `zeroize` |

See [`docs/decisions.md`](./docs/decisions.md) for the full rationale.

## Quick links

- [`docs/README.md`](./docs/README.md) — index of all internal docs
- [`docs/overview.md`](./docs/overview.md) — what / why / consumers
- [`docs/architecture.md`](./docs/architecture.md) — Rust core + bindings architecture
- [`docs/api.md`](./docs/api.md) — public `JovaWallet` API contract
- [`docs/chains.md`](./docs/chains.md) — supported chains and derivation paths
- [`docs/plan.md`](./docs/plan.md) — phased build plan
- [`docs/decisions.md`](./docs/decisions.md) — ADR-style design decisions

## Consumers

| Consumer | Binding | Status |
|---|---|---|
| iOS app | SwiftPM `JovaCore` | v1 launch target |
| Android app | Maven `io.jovachain:jova-core` | v1 launch target |
| Web wallet | npm `@jovachain/wallet-core` | Phase 6 |
| Backend services | Rust crate `jova-core` (or WASM) | Phase 6 |
| Hardware wallet firmware | `jova-core-primitives` (no_std) | Phase 7 |

## License

MIT.
