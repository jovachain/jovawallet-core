# Integration: Backend Services

How a backend service uses `jovawallet-core`. **Backend Rust direct (`cargo add jova-core`) is available continuously from v0.1.0 onward** — every release publishes the crates, so a Rust backend can adopt the SDK as soon as Phase 1 tags. Backend Node-via-WASM ships at v1.1.0 (Phase 6); see `integration-web.md` for the browser/Node WASM story.

Backends consume the SDK in one of three ways depending on language:

- **Rust backends**: depend on `jova-core` directly via `Cargo.toml`. Lowest overhead, full type safety.
- **Go backends**: invoke `jova-core` via cgo over a small C ABI exported by `jova-core-ffi`. Future, lower priority.
- **Node backends**: use the WASM npm package `@jovachain/wallet-core`. Same binding as the browser, runs server-side.

This document focuses on the Rust path (the recommended one). Node is identical to `integration-web.md` but server-side; Go is forward-looking.

## When to use the SDK on a backend

The SDK signs transactions. Most Jova backend services don't sign — they construct unsigned transactions and hand them to apps. But some flows want server-side signing:

- **Watch-only services**: derive addresses from a public xpub; the SDK supports this via the address-derivation path with public-only operations (Phase 7+).
- **Custodial helpers**: a backend that holds a hot wallet's mnemonic (in HSM or KMS) and signs sweeps or batched payouts.
- **Internal tooling**: signing test transactions, generating vectors, validating chain code without a phone in the loop.
- **Verification**: validating a signature received from the apps before relaying it.

If your backend doesn't need signing, you don't need the SDK. Most don't.

---

## Rust backend: adding the dependency

```toml
# Cargo.toml of your backend
[dependencies]
jova-core = "1.0"
```

The crate is published on `crates.io`. Pin to an exact version to match what your apps ship — drift between backend signing and app signing is the same hazard as drift between iOS and Android.

## The Hello-World

```rust
use jova_core::{JovaWallet, JovaChain, UnsignedTx, Strength};

fn main() -> anyhow::Result<()> {
    let mnemonic = jova_core::create_mnemonic(Strength::Bits256);
    let wallet = JovaWallet::from_mnemonic(&mnemonic)?;
    let address = wallet.address(JovaChain::Ethereum, 0)?;
    println!("ETH: {}", address.value);
    Ok(())
}
```

`JovaWallet` is `Drop`-managed; the seed is zeroized when it goes out of scope. No explicit close needed.

---

## Where the mnemonic lives

For a custodial backend, the mnemonic must live in a hardware-backed store:

- **AWS KMS / HSM**: The mnemonic (or the seed) is encrypted at rest under a CMK. Decrypt-on-use; never persist plaintext to disk.
- **HashiCorp Vault**: Same pattern; Vault's transit secret engine is well-suited.
- **GCP Cloud KMS / Azure Key Vault**: Equivalent.

The decrypt-and-sign pattern:

```rust
use jova_core::{JovaWallet, MnemonicBuffer};
use zeroize::Zeroizing;

async fn sign_payload(payload: UnsignedTx) -> anyhow::Result<SignedTx> {
    let plaintext: Zeroizing<Vec<u8>> = decrypt_from_kms("mnemonic-blob").await?.into();
    let buf = MnemonicBuffer {
        bytes: plaintext.to_vec(),
        passphrase: vec![],
    };
    // plaintext drops here, zeroizing
    let wallet = JovaWallet::from_mnemonic_buffer(buf)?;
    Ok(wallet.sign_tx(&payload)?)
}
```

The wallet is constructed per-request and dropped immediately after. The plaintext mnemonic exists for microseconds.

---

## Concurrency

`JovaWallet` is `Send` but not `Sync` — you can move it across threads but not share it concurrently. The pattern is:

- Fresh wallet per request (recommended).
- Or pin one wallet to one thread/task.

```rust
use tokio::task;

async fn handle_request(payload: UnsignedTx) -> Result<SignedTx, Error> {
    task::spawn_blocking(move || {
        // Construct fresh JovaWallet, sign, drop.
        sign_payload_sync(payload)
    }).await?
}
```

Signing is sync (ADR D10). Use `spawn_blocking` to avoid blocking the async runtime.

---

## Validating signatures from the apps

A common backend role: receiving a signed tx from an app and validating it before broadcasting.

For EVM:

```rust
use alloy_consensus::TxEnvelope;

fn validate_evm(signed_hex: &str, expected_from: &str) -> bool {
    let bytes = hex::decode(signed_hex.trim_start_matches("0x")).unwrap();
    let envelope: TxEnvelope = alloy_rlp::decode_exact(&bytes).unwrap();
    let recovered = envelope.recover_signer().unwrap();
    recovered.to_string().eq_ignore_ascii_case(expected_from)
}
```

For BTC, parse the tx, verify each input's witness against the expected public key.

For SOL, verify the ed25519 signature against the message bytes and the wallet's pubkey.

The SDK does not currently expose `verify(...)` methods directly — verification can be done with the underlying chain crates. Adding `verify(...)` to the public API is a Phase 7 candidate if multiple backends end up writing it themselves.

---

## Throughput

Indicative numbers (single-threaded, modern x86):

| Operation | Latency |
|---|---|
| `JovaWallet::from_mnemonic` | ~250 µs (PBKDF2-HMAC-SHA512 dominates) |
| `wallet.address(.ethereum)` | ~5 µs |
| `wallet.signTx(.evm)` | ~50 µs |
| `wallet.signTx(.bitcoin)` (PSBT, single input) | ~150 µs |
| `wallet.signTx(.solana)` | ~30 µs |

If your backend signs at high QPS, the bottleneck is `from_mnemonic` (PBKDF2 is intentionally slow). Consider:

- Cache derived seeds in process memory (`Arc<Zeroizing<Seed>>`). Construct `JovaWallet::from_seed(seed)` (Phase 6 API addition) instead of from mnemonic on every request.
- Pin one wallet per worker thread, signing many requests with one wallet.

The cache must be guarded — exposing the seed in process memory is a much higher-blast-radius decision than ephemeral wallets. Threat-model accordingly.

---

## Logging

The SDK does no logging. The backend logs the SDK's outputs. Same rules as on phones:

| Safe | Never |
|---|---|
| `JovaError` variant + reason | `mnemonic.words` |
| `chain` involved | `address.value` (PII) |
| SDK version | `signed.rawHex` (PII once linked) |
| Signing latency | seed bytes |

---

## Testing the backend's use of the SDK

The backend should have its own integration tests that exercise the signing path against `spec/test-vectors.json`. The same vectors that validate the iOS and Android bindings validate the backend.

```rust
#[cfg(test)]
mod tests {
    use jova_core::{JovaWallet, UnsignedTx};

    #[test]
    fn signs_eip1559_per_spec_vector() {
        let vectors: TestVectors = serde_json::from_str(
            include_str!("../../jovawallet-core/spec/test-vectors.json")
        ).unwrap();

        let v = vectors.vector("evm.tx.eip1559_simple_transfer");
        let wallet = JovaWallet::from_mnemonic(&v.input.mnemonic).unwrap();
        let signed = wallet.sign_tx(&v.input.unsigned_tx).unwrap();
        assert_eq!(signed.raw_hex, v.expected.signed_hex);
    }
}
```

Vector parity with apps is non-negotiable: a backend that produces a different signature than the apps for the same input is broken.

---

## Node backend (WASM)

Same as `integration-web.md`. The Node target initializes automatically; otherwise the API is identical.

```typescript
import { JovaWallet, UnsignedTx } from '@jovachain/wallet-core';

const mnemonic = await loadMnemonicFromVault();
const wallet = JovaWallet.fromMnemonicBuffer({
    bytes: new TextEncoder().encode(mnemonic),
    passphrase: new Uint8Array(),
});
try {
    const signed = wallet.signTx(payload);
    return signed.rawHex;
} finally {
    wallet.destroy();
}
```

---

## Go backend (forward-looking)

Phase 7+. The plan is to expose a stable C ABI from `jova-core-ffi` and a thin Go wrapper. Until that ships, Go backends should use the WASM build via a Node sidecar or call out to a Rust microservice.

---

## Don'ts

- Don't store the mnemonic plaintext on the backend's disk.
- Don't share `JovaWallet` across goroutines / async tasks / threads.
- Don't construct `JovaWallet` per request without seed caching at high QPS — PBKDF2 will become the bottleneck.
- Don't add a custom verification layer that re-derives the address differently than the SDK does. Use the SDK's address derivation as the canonical source.
- Don't log signed-tx hex.
