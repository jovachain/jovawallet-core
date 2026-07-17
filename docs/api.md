# Public API — `JovaWallet`

This is the public API the iOS app, Android app, web wallet, backend services, and hardware-wallet firmware all depend on. Every binding implements it identically; every implementation is validated against `spec/test-vectors.json` in CI.

The shapes shown here are language-agnostic. Each binding translates them to idiomatic types in its language:

- **Swift:** `enum` with associated values, `Result` / `throws`, `final class` for resource handles.
- **Kotlin:** `sealed class` for sum types, `Result<T>` / exceptions, `AutoCloseable` for resource handles.
- **JavaScript:** discriminated-union objects with `kind` tag, `Promise<Result>`, `destroy()` for resource handles.
- **Rust:** plain `enum`, `Result<T, JovaError>`, `Drop` for resource handles.

Method names and parameter orders **match across every binding**. If they ever drift, the binding is wrong, not the spec.

---

## Types

### `Mnemonic`

A BIP-39 mnemonic phrase. Stored as a string when crossing the API boundary.

```
Mnemonic {
  words: String       // space-separated, normalized to NFKD, lowercased
  passphrase: String  // BIP-39 passphrase, "" if none
}
```

For host-controlled secret clearing, see `MnemonicBuffer` below.

### `MnemonicBuffer` (escape hatch for clearable input)

Some apps want to feed the mnemonic in as a byte buffer they own and can clear. This variant accepts the words as `Vec<u8>` (UTF-8) instead of `String`. The SDK never copies it longer than it needs.

```
MnemonicBuffer {
  bytes: Bytes        // UTF-8 of normalized words
  passphrase: Bytes   // UTF-8 of passphrase
}
```

Apps free to ignore this in v1; defaulting to `Mnemonic` is fine. Documented in `memory-and-keys.md`.

### `JovaChain`

The set of chains the SDK supports.

```
JovaChain =
  | ethereum
  | polygon
  | bsc
  | arbitrum
  | optimism
  | base
  | bitcoin
  | solana
  | xrp
  | customEvm(chainId: UInt64)   // for the future Jova chain or any other EVM
```

Future variants will follow the additive rule (D8): adding a chain is a minor version bump. See `chains.md` for derivation paths and address formats.

### `Strength`

```
Strength =
  | bits128   // 12 words
  | bits256   // 24 words
```

### `Address`

```
Address {
  chain: JovaChain
  value: String         // canonical string form per chain (EIP-55, base58, bech32, etc.)
}
```

### `UnsignedTx`

A discriminated union over the supported transaction families. Apps build these from backend responses; SDKs do not construct them.

```
UnsignedTx =
  | evm({
      chainId: UInt64
      nonce: UInt64
      to: String                         // 0x-prefixed
      value: String                      // wei, decimal string
      gasLimit: UInt64
      maxFeePerGas: String               // wei, decimal string
      maxPriorityFeePerGas: String       // wei, decimal string
      data: String                       // 0x-prefixed hex, "0x" if empty
      accessList: AccessList?            // optional EIP-2930 access list
    })
  | bitcoin({
      psbtBase64: String                 // BIP-174 PSBT
    })
  | solana({
      messageBase64: String              // serialized v0 transaction message
      recentBlockhash: String
    })
  | xrp({
      txJson: String                     // canonical XRP tx JSON
    })

AccessList = [{ address: String, storageKeys: [String] }]
```

Wei values are decimal strings to avoid 64-bit overflow on chains that have larger fees than `UInt64::MAX` (already an issue on some L2s during congestion).

### `SignedTx`

```
SignedTx {
  chain: JovaChain
  rawHex: String       // 0x-prefixed for EVM, hex for others; ready for broadcast
  txHash: String       // canonical hash for the chain (keccak for EVM, sha256d for BTC...)
}
```

`txHash` is convenience — the value is computable from `rawHex`. Including it on the SDK side ensures every binding computes it the same way (some clients have famously gotten this wrong).

### `SignableMessage`

```
SignableMessage =
  | evmPersonalSign({ message: String })                            // EIP-191
  | evmTypedDataV4({ json: String })                                // EIP-712 v4
  | solana({ messageBase64: String })                               // raw bytes to sign
  | bitcoin({ message: String, address: String, scheme: BtcMsgScheme })

BtcMsgScheme =
  | bip322
  | legacy   // signMessage as in Bitcoin Core's RPC
```

### `Signature`

```
Signature {
  hex: String          // canonical encoding for the chain (rsv for EVM, base58 for SOL, etc.)
}
```

### `JovaError`

The exhaustive error taxonomy. See `error-model.md` for per-variant semantics and per-binding mapping.

```
JovaError =
  | invalidMnemonic                          // failed BIP-39 wordlist or checksum
  | invalidPassphrase
  | invalidDerivationPath(reason)
  | invalidAddress(chain)
  | unsupportedChain(JovaChain)
  | malformedUnsignedTx(reason)
  | malformedSignableMessage(reason)
  | signingFailed(reason)
  | internal(reason)                         // crate-level invariant violation; bug
```

Every error carries `chain` or `reason` context where applicable so consumers can produce useful UX without parsing strings.

---

## Static / class methods

```
static createMnemonic(strength: Strength = .bits128) -> Mnemonic
//   Generates a fresh BIP-39 mnemonic using the OS CSPRNG.
//   Strength.bits128 → 12 words, Strength.bits256 → 24 words.

static isValidMnemonic(words: String, passphrase: String = "") -> Bool
//   True iff words is a valid BIP-39 phrase whose checksum matches.
//   Whitespace and case are normalized before checking.

static isValidAddress(string: String, on: JovaChain) -> Bool
//   True iff string parses as a canonical address on the given chain.
//   For customEvm(N), accepts any EIP-55-checksummed 0x address.
```

---

## Instance methods

A `JovaWallet` is constructed from a mnemonic and is the entry point for signing. Apps build one when they need to sign and let it deinit/close immediately after.

```
init(mnemonic: Mnemonic) throws(JovaError)
//   throws .invalidMnemonic if checksum or wordlist fails

init(mnemonic: MnemonicBuffer) throws(JovaError)
//   same, but takes a clearable byte buffer

address(on: JovaChain, account: UInt32 = 0) throws(JovaError) -> Address
//   Derives the canonical address for the given chain and account index.

addresses(on: [JovaChain], account: UInt32 = 0) throws(JovaError) -> [Address]
//   Convenience: derive multiple addresses in one call. Order matches the input.

signTx(unsigned: UnsignedTx, account: UInt32) throws(JovaError) -> SignedTx
//   Chain is implicit in the `UnsignedTx` variant; for EVM, the chain ID inside
//   the variant payload is authoritative. SDK routes to the right signer.
//   Signs with the key at HD `account` — the same key `address(on:account:)`
//   returns for that chain and account. `account` is required; pass 0 for the
//   primary account.

signMessage(msg: SignableMessage, account: UInt32) throws(JovaError) -> Signature
//   Chain is implicit in the `SignableMessage` variant. Signs with the key at
//   HD `account`, matching `address(on:account:)`. `account` is required.
```

> **Account index** selects the HD account (MetaMask-style multiple accounts from one mnemonic). `address`, `signTx`, and `signMessage` all take it and derive the same key for a given `(chain, account)`. For EVM it increments the BIP-44 `address_index` (`m/44'/60'/0'/0/N`), matching MetaMask; see `chains.md` for the per-chain path. It is a **required** parameter on the signing calls (UniFFI's Kotlin bindings don't emit default argument values); single-account apps pass `0`.

---

## Memory and lifecycle

- `JovaWallet` holds an internal handle (a Rust-side `Box<JovaWallet>`). Bindings translate this to:
  - **Swift**: `final class` with `deinit` calling Rust-side `clear()`.
  - **Kotlin**: `class : AutoCloseable`. Apps `use { … }` to bound the lifetime.
  - **JavaScript**: instance with explicit `.destroy()` method; documented requirement to call it.
  - **Rust**: `Drop` clears via `zeroize`.
- Method calls are pure: no global state, no caching at the SDK level.
- Repeated derivation of the same address on the same `JovaWallet` is allowed and cheap (key derivation is microseconds).
- **Concurrent calls on the same instance are not safe.** Apps should construct a fresh `JovaWallet` per signing operation; do not share across threads.

See `memory-and-keys.md` for the full secret-clearing contract.

---

## What's NOT on this API

Deliberately. If you find yourself wanting one of these, the answer is "another module, not this one":

- `broadcast(...)` — backend's job (`jova-rpc`).
- `getBalance(...)` — backend's job.
- `estimateFee(...)` — backend's job.
- `connectToWalletConnect(...)` — app's job; once an unsigned payload arrives, hand it to `signTx(unsigned:)`.
- `saveSeedPhrase(...)` / `loadSeedPhrase(...)` — apps own Keychain/Keystore.
- `setLogLevel(...)` — bindings own their logging stack; the SDK does not log.
- Async variants of `sign(...)` — see ADR D10. Bindings wrap the sync call in their preferred async primitive.

---

## Versioning

- **v0.x:** API may change between minor versions while we settle the contract.
- **v1.0:** API is locked. Breaking changes only in major versions.
- **Adding a new chain** is a minor version bump (additive enum variant).
- **Adding a new method** is a minor version bump.
- **Adding a new error variant** is a minor version bump (consumers must use exhaustive `_:` defaults).
- **Removing or changing a method** is a major version bump.

Every binding ships at the same tag (ADR D8). The Rust crate versions move in lockstep with the Swift / Kotlin / npm package versions.

---

## Stability checklist for v1

Before tagging `v1.0.0`:

1. Every method documented above has been on the API surface for at least one minor release without modification.
2. Every chain in `JovaChain` has at least three vector triplets in `spec/test-vectors.json` covering address derivation, transaction signing, and message signing.
3. Every binding's `ApiSurfaceTests` confirms every method exists with the documented signature.
4. The Swift, Kotlin, and JavaScript surfaces have each been used end-to-end by an example app shipping in `examples/`.
5. Public API changes between v1.0 and the v0.x predecessor are documented in `CHANGELOG.md` so app teams can plan migration.
