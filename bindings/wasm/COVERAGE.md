# WASM Chain Coverage

The npm package compiles a subset of `JovaWallet`'s chains to WebAssembly.
This file documents the v1.1.0 reality of the `@jovachain/wallet-core`
package and is the authoritative source of truth for WASM coverage.

## Status matrix (Phase 6 / v1.1.0)

| Chain | WASM status | Notes |
|---|---|---|
| Ethereum / Polygon / BSC / Arbitrum / Optimism / Base / customEvm | Full | `alloy` 2.x is WASM-clean with `getrandom@0.2` set to the `js` backend. |
| Solana | Full | Anza split crates (`solana-keypair` / `solana-transaction` / `solana-message` / `solana-pubkey` / `solana-signature`) compile cleanly to `wasm32-unknown-unknown`. |
| Bitcoin | **Deferred** | `bdk_wallet` + the `secp256k1-sys` C crate emit code paths that need additional hardening before they are trusted in untrusted browser contexts. Recorded user decision 2026-05-11: ship WASM v1.1 without BTC signing. Calling `signTx` / `signMessage` for Bitcoin returns `JovaError.UnsupportedChain` at runtime. |
| XRP | **Deferred** | The `xrpl-rust` 1.x signing path was not validated in browsers as of the Phase 6 freeze; XRP returns `JovaError.UnsupportedChain` at runtime to match the Bitcoin policy. Recorded user decision 2026-05-11. |

## What "deferred" means

The runtime error path:

```typescript
import { JovaWallet, JovaException } from '@jovachain/wallet-core';

try {
  wallet.signTx({ kind: 'bitcoin', psbtBase64: '...' });
} catch (e) {
  if (e instanceof JovaException && e.error.kind === 'unsupportedChain') {
    // e.error.chain is "bitcoin" or "xrp"
  }
}
```

Bitcoin and XRP **are fully supported** in:

- The Rust core (`jova-core` with `features = ["chains"]`).
- The Swift binding (iOS).
- The Kotlin binding (Android).

If your application needs BTC or XRP signing in a browser/Node environment
today, the recommended workaround is to run signing on a backend service
that uses the Rust crate directly, or to use the native Swift / Kotlin
bindings in a mobile context.

## Why the BTC / XRP cutout exists

- `bdk_wallet`'s wasm story is non-trivial; the engine pulls a substantial
  amount of code that meaningfully impacts bundle size and surface area.
- The `xrpl-rust` crate has not been audited in this project's threat model
  for browser use; we'd rather not ship it than ship it incorrectly.

These restrictions will be revisited in a future minor release if and when
the underlying crates have a clean WASM story and a documented audit trail.

## Feature flags

`crates/jova-core-wasm` declares feature flags as documentation of intent:

- `chain-evm` (default): EVM family signing.
- `chain-sol` (default): Solana signing.
- `chain-btc` / `chain-xrp` (not in default): reserved; the WASM layer
  rejects these variants at runtime regardless of build configuration.

The flags do **not** gate compilation today — the chains crate as a whole
still compiles in. Runtime rejection of Bitcoin / XRP variants happens in
`crates/jova-core-wasm/src/lib.rs` before any chain code executes.
