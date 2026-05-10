# Error Model

The complete error taxonomy for `jovawallet-core`, what each variant means, and how it surfaces in every binding.

## Principles

1. **Exhaustive at the source.** `JovaError` is a closed enum in Rust. Every fallible operation returns `Result<T, JovaError>`. Bindings translate the enum into idiomatic language types.
2. **Structured, not stringly.** Every variant carries enough structured context that consumer UX code can branch on the variant without parsing strings. Strings are for the developer console only.
3. **Stable variants.** Adding a new variant is a minor version bump. Removing or renaming one is a major version bump. Apps that handle errors with exhaustive `switch` / `when` get compile-time warnings on minor upgrades.
4. **No secrets in errors.** No error variant ever carries mnemonic words, seed bytes, or private-key bytes — even in a `reason` string. The Rust code uses `#[derive(thiserror::Error)]` with carefully crafted `Display` impls that omit secret context.

---

## The taxonomy

```rust
#[derive(Debug, thiserror::Error)]
pub enum JovaError {
    /// BIP-39 wordlist or checksum failed.
    #[error("invalid mnemonic")]
    InvalidMnemonic,

    /// Passphrase failed normalization or contained a forbidden character.
    #[error("invalid passphrase")]
    InvalidPassphrase,

    /// Derivation path could not be parsed or is malformed.
    #[error("invalid derivation path: {reason}")]
    InvalidDerivationPath { reason: String },

    /// Address fails per-chain validation (length, checksum, prefix).
    #[error("invalid address for {chain:?}")]
    InvalidAddress { chain: JovaChain },

    /// Chain isn't currently supported by this build of the SDK.
    #[error("unsupported chain: {0:?}")]
    UnsupportedChain(JovaChain),

    /// Unsigned transaction was malformed for its declared variant.
    #[error("malformed unsigned tx: {reason}")]
    MalformedUnsignedTx { reason: String },

    /// Signable message was malformed.
    #[error("malformed signable message: {reason}")]
    MalformedSignableMessage { reason: String },

    /// Signing operation failed at the cryptographic layer (rare; typically a
    /// chain-crate-internal error such as a bad chainId or invalid PSBT input).
    #[error("signing failed: {reason}")]
    SigningFailed { reason: String },

    /// SDK internal invariant violated. This is a bug in jovawallet-core.
    /// Apps should not catch this specifically; it indicates an SDK fault.
    #[error("internal SDK error: {reason}")]
    Internal { reason: String },
}
```

Below: every variant, when it fires, examples, and what apps should do.

### `invalidMnemonic`

| | |
|---|---|
| **When** | Words don't match the BIP-39 wordlist, *or* checksum bits don't validate, *or* word count isn't 12/15/18/21/24. |
| **Examples** | `"apple banana cherry ..."`, `"abandon ... abandon abandon abxut"` (typo). |
| **App UX** | Reject input, prompt the user to retype. Do *not* echo which word was wrong (no information leakage to a screen-shoulder-surfer). |
| **Reason field** | None — variant alone is enough. |

### `invalidPassphrase`

| | |
|---|---|
| **When** | Passphrase normalization (NFKD) fails or contains an unsupported control sequence. Empty passphrase is always valid. |
| **Examples** | A passphrase with embedded null bytes, malformed UTF-8 from a host-language `String` that crossed FFI corrupted. |
| **App UX** | Almost never fires from a typed passphrase. If it does, sanitize the input. |

### `invalidDerivationPath { reason }`

| | |
|---|---|
| **When** | Parsing a string-form derivation path fails. *Internal use only* in v1 — we don't expose path strings on the public API. |
| **Examples** | `"m/44'/60'"` (too short for the canonical form), `"x/y/z"` (not BIP-32 syntax). |
| **App UX** | Should not surface from any v1 public-API call. If it does, file a bug. |

### `invalidAddress { chain }`

| | |
|---|---|
| **When** | A string fails validation for the given chain. Used by `isValidAddress` to return `false` *and* by signing flows that receive a `to:` field with a malformed value. |
| **Examples** | `isValidAddress("bc1q...invalidchecksum", on: .bitcoin)` returns `false`. `sign(.evm({ to: "0xnotanaddress" }))` throws `invalidAddress(.ethereum)`. |
| **App UX** | Inline form validation. Reject the input, indicate which field is bad. |

### `unsupportedChain(JovaChain)`

| | |
|---|---|
| **When** | A `JovaChain` value the SDK doesn't currently support is used. Possible if an older binding sees a newer enum value (e.g., a future chain a backend already knows about). |
| **Examples** | App sends `.zksync` to a v0.4 SDK that only knows about `.ethereum`. |
| **App UX** | Tell the user this version of the app needs an update. |

### `malformedUnsignedTx { reason }`

| | |
|---|---|
| **When** | The `UnsignedTx` payload doesn't decode or fails per-chain semantic checks. |
| **Examples** | `reason: "psbt_invalid_base64"`, `reason: "evm_decimal_string_overflow"`, `reason: "sol_blockhash_mismatch"`, `reason: "evm_chainid_mismatch"`, `reason: "legacy_tx_not_supported"`. |
| **Reason vocabulary** | Stable strings — listed in `spec/errors.md`. Apps may pattern-match on them but should default-handle the unknown case. |
| **App UX** | Backend bug; show a generic "transaction couldn't be prepared" and report telemetry. |

### `malformedSignableMessage { reason }`

| | |
|---|---|
| **When** | A `SignableMessage` doesn't parse or violates the scheme it claims. |
| **Examples** | `reason: "eip712_typed_data_invalid_json"`, `reason: "eip712_missing_domain"`, `reason: "btc_message_address_mismatch"`. |
| **App UX** | Same as above — usually a bug upstream of the SDK. |

### `signingFailed { reason }`

| | |
|---|---|
| **When** | The signing primitive itself failed. Rare. Almost always indicates upstream-data malformation that slipped past validation. |
| **Examples** | A PSBT input claims the wallet's key but the script doesn't match anything we can sign; alloy's signer rejects a request because hashing returned `None`. |
| **App UX** | Show "couldn't sign this transaction" and log telemetry with the reason field. |

### `internal { reason }`

| | |
|---|---|
| **When** | An invariant inside the SDK was violated. This is a bug in `jovawallet-core` itself. |
| **Examples** | A `match` that should be exhaustive returned a default branch; a buffer length didn't match an asserted size. |
| **App UX** | Treat as an unrecoverable error. Log telemetry with full reason. File an issue. |

---

## Per-binding mapping

### Swift

`JovaError` translates to a Swift `enum` with associated values, conforming to `Error`:

```swift
public enum JovaError: Error, Equatable {
    case invalidMnemonic
    case invalidPassphrase
    case invalidDerivationPath(reason: String)
    case invalidAddress(chain: JovaChain)
    case unsupportedChain(JovaChain)
    case malformedUnsignedTx(reason: String)
    case malformedSignableMessage(reason: String)
    case signingFailed(reason: String)
    case `internal`(reason: String)
}
```

Methods throw it: `func signTx(unsigned: UnsignedTx) throws -> SignedTx`. Use Swift's typed `throws(JovaError)` once that lands in stable.

App pattern:

```swift
do {
    let signed = try wallet.signTx(unsigned: tx)
    return signed
} catch JovaError.invalidMnemonic {
    showRetry("Mnemonic is invalid.")
} catch let JovaError.malformedUnsignedTx(reason) {
    log("backend bug: \(reason)")
    showGeneric()
} catch {
    log("unknown SDK error: \(error)")
    showGeneric()
}
```

### Kotlin

`JovaError` translates to a Kotlin `sealed class`, with each variant a `data class` carrying its fields. Method failures throw `JovaException(error: JovaError)`.

```kotlin
sealed class JovaError {
    data object InvalidMnemonic : JovaError()
    data object InvalidPassphrase : JovaError()
    data class InvalidDerivationPath(val reason: String) : JovaError()
    data class InvalidAddress(val chain: JovaChain) : JovaError()
    data class UnsupportedChain(val chain: JovaChain) : JovaError()
    data class MalformedUnsignedTx(val reason: String) : JovaError()
    data class MalformedSignableMessage(val reason: String) : JovaError()
    data class SigningFailed(val reason: String) : JovaError()
    data class Internal(val reason: String) : JovaError()
}

class JovaException(val error: JovaError) : Exception(error.toString())
```

App pattern:

```kotlin
try {
    val signed = wallet.signTx(tx)
    handleSigned(signed)
} catch (e: JovaException) {
    when (val err = e.error) {
        is JovaError.InvalidMnemonic -> showRetry("Mnemonic is invalid.")
        is JovaError.MalformedUnsignedTx -> {
            log("backend bug: ${err.reason}")
            showGeneric()
        }
        else -> {
            log("unknown SDK error: $err")
            showGeneric()
        }
    }
}
```

### JavaScript / TypeScript

WASM bindings throw `JovaError` JS-class instances with a discriminator field:

```typescript
export type JovaError =
  | { kind: 'invalidMnemonic' }
  | { kind: 'invalidPassphrase' }
  | { kind: 'invalidDerivationPath'; reason: string }
  | { kind: 'invalidAddress'; chain: JovaChain }
  | { kind: 'unsupportedChain'; chain: JovaChain }
  | { kind: 'malformedUnsignedTx'; reason: string }
  | { kind: 'malformedSignableMessage'; reason: string }
  | { kind: 'signingFailed'; reason: string }
  | { kind: 'internal'; reason: string };

export class JovaException extends Error {
  constructor(public readonly error: JovaError) {
    super(JSON.stringify(error));
    this.name = 'JovaException';
  }
}
```

App pattern:

```typescript
try {
  const signed = await wallet.signTx(tx);
  return signed;
} catch (e) {
  if (e instanceof JovaException) {
    switch (e.error.kind) {
      case 'invalidMnemonic': return showRetry();
      case 'malformedUnsignedTx': {
        log('backend bug:', e.error.reason);
        return showGeneric();
      }
      default: return showGeneric();
    }
  }
  throw e;
}
```

### Direct Rust

Same as the canonical definition. Use `?` to propagate, `match` to handle.

---

## Reason-string vocabulary

Reason strings on `malformedUnsignedTx`, `malformedSignableMessage`, and `signingFailed` are part of the public API. They are documented exhaustively in `spec/errors.md` and never change in patch releases. New reasons are minor-version bumps.

### `malformedUnsignedTx`

| Reason | Means |
|---|---|
| `psbt_invalid_base64` | BTC: PSBT base64 decode failed |
| `psbt_unsupported_version` | BTC: PSBT v2 features not yet supported |
| `psbt_no_signable_inputs` | BTC: none of the PSBT inputs are signable by this wallet |
| `evm_chainid_mismatch` | EVM: variant payload chainId disagrees with what `customEvm(N)` declared |
| `evm_decimal_string_overflow` | EVM: a numeric field's decimal string exceeds U256 max |
| `evm_decimal_string_invalid` | EVM: a numeric field isn't a decimal integer |
| `evm_to_address_invalid` | EVM: `to` field isn't a checksummed address |
| `evm_data_not_hex` | EVM: `data` field isn't 0x-prefixed hex |
| `legacy_tx_not_supported` | EVM: legacy (type-0) tx received; we only sign EIP-1559 |
| `sol_blockhash_mismatch` | SOL: outer `recentBlockhash` disagrees with the message blockhash |
| `sol_invalid_base64` | SOL: base64 decode of message failed |
| `sol_message_unsupported_version` | SOL: only v0 supported |
| `xrp_invalid_json` | XRP: tx JSON didn't parse |
| `xrp_missing_required_field` | XRP: required field absent (e.g., `Account`, `TransactionType`) |

### `malformedSignableMessage`

| Reason | Means |
|---|---|
| `eip712_typed_data_invalid_json` | EVM: typed-data JSON didn't parse |
| `eip712_missing_domain` | EVM: typed data has no `domain` |
| `eip712_unknown_type` | EVM: typed data references a struct that isn't defined |
| `btc_message_address_mismatch` | BTC: address doesn't correspond to wallet's derivation |
| `btc_unsupported_scheme` | BTC: unknown `BtcMsgScheme` value |

### `signingFailed`

| Reason | Means |
|---|---|
| `secp256k1_signing_error` | secp256k1 returned an error during sign |
| `ed25519_signing_error` | ed25519-dalek returned an error |
| `psbt_finalize_failed` | bdk_wallet rejected the finalized PSBT |
| `xrp_serialize_failed` | XRPL canonical serialization rejected the tx |

---

## What errors must never surface

Some failure modes are designed out of existence:

- **Out-of-memory on signing.** Signing operations preallocate fixed buffers; a sign call cannot fail because of allocation pressure on a phone with a few MB free.
- **Network errors.** The SDK does no I/O. A network error from the SDK indicates a bug; would surface as `internal { reason: "unexpected_io" }`.
- **Timing-related errors.** Operations are synchronous and bounded.
- **Threading errors.** The SDK is not thread-safe across operations on a single instance — but that's a documented constraint, not an error path. Callers who violate it get undefined behavior, not a clean error.

---

## Telemetry and logging

The SDK does **no logging itself.** Bindings receive errors and decide what to log. This keeps the SDK from depending on a logging framework and makes it auditable in isolation.

For diagnostic-grade telemetry, apps should log:

- Variant name (e.g., `malformedUnsignedTx`)
- `reason` if present
- `chain` if present
- The Git SHA / version of the SDK build (exposed as `JovaCore.version` static)

Apps must **never** log:

- Mnemonic words
- Address values from the wallet (PII)
- Raw signed-tx hex (PII; can be reconstructed by anyone with the key)

`security.md` includes a checklist for telemetry-safe logging.
