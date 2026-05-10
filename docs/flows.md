# Flows

Sequence diagrams for every public operation. The diagrams trace a call from the host application, through the binding, into the Rust core, and back out — showing where each component does its work and what crosses the FFI boundary.

The same flow applies to every binding (Swift, Kotlin, JS, direct Rust). The diagrams use Swift naming for concreteness.

---

## 1. Generating a mnemonic

```
App                    Binding                   jova-core              jova-core-primitives
 │                        │                         │                         │
 │  createMnemonic(.bits256)                        │                         │
 │ ───────────────────────►                         │                         │
 │                        │ ffi: jova_create_mnemonic(strength=256)           │
 │                        │ ───────────────────────►│                         │
 │                        │                         │ Mnemonic::generate(256) │
 │                        │                         │ ───────────────────────►│
 │                        │                         │     │                   │
 │                        │                         │     │ OS CSPRNG: 32 B   │
 │                        │                         │     │ → bip39 wordlist  │
 │                        │                         │     │ → 24-word string  │
 │                        │                         │     │                   │
 │                        │                         │ ◄───────────────────────│
 │                        │ ◄────────────────────── │ Mnemonic{ words, "" }   │
 │                        │ marshal: Rust String → Swift String               │
 │ ◄──────────────────────                          │                         │
 │  Mnemonic                                                                  │
```

**Notes**

- The OS CSPRNG is OS-provided: `getrandom(2)` on Linux, `SecRandomCopyBytes` on Apple, `BCryptGenRandom` on Windows. Rust's `getrandom` crate wraps these.
- The mnemonic is *generated* in Rust memory and *copied* across FFI as a String. There is no way to keep it Rust-side because the entire point of the call is to give the words to the user.

---

## 2. Constructing a `JovaWallet`

```
App                    Binding                   jova-core           jova-core-primitives
 │                        │                         │                       │
 │  JovaWallet(mnemonic: m)                         │                       │
 │ ───────────────────────►                         │                       │
 │                        │ ffi: jova_wallet_from_mnemonic(words, passphrase)
 │                        │ ───────────────────────►│                       │
 │                        │                         │ Mnemonic::validate    │
 │                        │                         │ ─────────────────────►│
 │                        │                         │ ◄───── Result          │
 │                        │                         │  if invalid: throw     │
 │                        │                         │                        │
 │                        │                         │ Mnemonic::to_seed     │
 │                        │                         │ ─────────────────────►│
 │                        │                         │ ◄───── Seed (Zeroizing)│
 │                        │                         │                        │
 │                        │                         │ build internal handle  │
 │                        │                         │ Box<JovaWalletInner>   │
 │                        │                         │ holding the Seed       │
 │                        │ ◄────────────────────── │ Arc<JovaWalletInner>   │
 │                        │ wrap in Swift class     │                        │
 │ ◄──────────────────────                          │                        │
 │  wallet (handle)                                                          │
```

**Notes**

- The seed is held inside the internal handle. It never leaves Rust memory.
- The Swift `JovaWallet` is a thin `final class` whose `deinit` calls back into Rust to drop the handle. Drop in Rust zeroizes the seed via `zeroize::Zeroizing<[u8; 64]>`.
- An invalid mnemonic short-circuits before any seed is allocated. The error returned is `JovaError.invalidMnemonic`.

---

## 3. Address derivation

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.address(on: .ethereum)                 │                         │
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_address(handle, chain=Ethereum, account=0)
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive XPrv at m/44'/60'/0'/0/0
 │                    │                         │ (jova-core-primitives)  │
 │                    │                         │                         │
 │                    │                         │ chains::evm::EvmSigner::derive_address
 │                    │                         │ ───────────────────────►│
 │                    │                         │  pubkey → keccak256[12:]│
 │                    │                         │  → EIP-55 checksum      │
 │                    │                         │ ◄───────────────────────│
 │                    │                         │ Address{ chain, value } │
 │                    │ ◄────────────────────── │                         │
 │ ◄──────────────────                          │                         │
 │ Address("0xAbC1...")                                                    │
```

**Notes**

- BIP-32 derivation is fast (microseconds). Derived `XPrv` is local to the call and zeroized when the call returns.
- The path is dictated by the chain (see `chains.md`); the SDK does not let the app pick arbitrary paths. Apps that need account discovery use `account: UInt32`.

---

## 4. EVM transaction signing (EIP-1559)

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.signTx(unsigned: .evm(...))            │                         │
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_sign_tx(handle, unsigned: UnsignedTx)   │
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive XPrv at m/44'/60'/0'/0/0
 │                    │                         │                         │
 │                    │                         │ dispatch to chains::evm  │
 │                    │                         │ ───────────────────────►│
 │                    │                         │  build alloy::TxEip1559 │
 │                    │                         │  hash with EIP-1559 prefix│
 │                    │                         │  secp256k1.sign(hash)   │
 │                    │                         │  recover y-parity → v   │
 │                    │                         │  serialize: 0x02 || rlp │
 │                    │                         │  compute keccak tx hash │
 │                    │                         │ ◄───────────────────────│
 │                    │                         │ SignedTx{ chain, rawHex,│
 │                    │                         │           txHash }      │
 │                    │ ◄────────────────────── │                         │
 │                    │ marshal SignedTx to Swift                          │
 │ ◄──────────────────                          │                         │
 │ SignedTx                                                                │
```

---

## 5. Bitcoin PSBT signing

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.signTx(unsigned: .bitcoin(psbtBase64: ...))
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_sign_tx(handle, unsigned: UnsignedTx)   │
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive XPrv at m/84'/0'/0'/0/0
 │                    │                         │                         │
 │                    │                         │ dispatch to chains::btc  │
 │                    │                         │ ───────────────────────►│
 │                    │                         │  base64 → bdk::Psbt     │
 │                    │                         │  for each input:        │
 │                    │                         │   if our key → sign     │
 │                    │                         │   else: leave           │
 │                    │                         │  finalize signable parts│
 │                    │                         │  if all inputs signed:  │
 │                    │                         │   serialize tx          │
 │                    │                         │   compute sha256d hash  │
 │                    │                         │   return SignedTx       │
 │                    │                         │  else:                  │
 │                    │                         │   return updated PSBT   │
 │                    │                         │   in rawHex (caller     │
 │                    │                         │   coordinates next      │
 │                    │                         │   signer)               │
 │                    │                         │ ◄───────────────────────│
 │                    │ ◄────────────────────── │ SignedTx                │
 │ ◄──────────────────                          │                         │
 │ SignedTx                                                                │
```

**Notes**

- The PSBT may contain inputs whose keys this wallet doesn't own (multi-party PSBT). The SDK signs only what it can sign. The "is the result a complete tx or an updated PSBT?" decision is encoded in `SignedTx.rawHex` — apps inspect it.
- `bdk_wallet` handles input identification (script type, derivation path) internally. We don't reinvent that.

---

## 6. Solana versioned-tx signing

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.signTx(unsigned: .solana(messageBase64, recentBlockhash))
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_sign_tx(handle, unsigned: UnsignedTx)   │
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive ed25519 keypair  │
 │                    │                         │ at m/44'/501'/0'/0' via │
 │                    │                         │ SLIP-10 from seed       │
 │                    │                         │                         │
 │                    │                         │ dispatch to chains::sol │
 │                    │                         │ ───────────────────────►│
 │                    │                         │  base64 → MessageV0     │
 │                    │                         │  patch recent_blockhash │
 │                    │                         │  ed25519.sign(message)  │
 │                    │                         │  build VersionedTx{     │
 │                    │                         │    signatures: [sig],   │
 │                    │                         │    message              │
 │                    │                         │  }                      │
 │                    │                         │  serialize wire format  │
 │                    │                         │ ◄───────────────────────│
 │                    │ ◄────────────────────── │ SignedTx                │
 │ ◄──────────────────                          │                         │
 │ SignedTx                                                                │
```

**Notes**

- Solana derivation uses SLIP-10, not BIP-32 (BIP-32 doesn't apply to ed25519). `slip-10` crate, `no_std`-clean.
- The `recentBlockhash` parameter is a sanity check — if the message already encodes a different blockhash, the SDK errors with `malformedUnsignedTx("blockhash_mismatch")`. Backend should send consistent values.

---

## 7. XRP transaction signing

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.signTx(unsigned: .xrp(txJson))         │                         │
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_sign_tx(handle, unsigned: UnsignedTx)   │
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive secp256k1 key    │
 │                    │                         │ at m/44'/144'/0'/0/0    │
 │                    │                         │                         │
 │                    │                         │ dispatch to chains::xrp │
 │                    │                         │ ───────────────────────►│
 │                    │                         │  parse JSON → tx struct │
 │                    │                         │  inject SigningPubKey   │
 │                    │                         │  serialize canonical    │
 │                    │                         │  prepend tx prefix      │
 │                    │                         │  sha512_half digest     │
 │                    │                         │  secp256k1 DER sign     │
 │                    │                         │  inject TxnSignature    │
 │                    │                         │  re-serialize           │
 │                    │                         │ ◄───────────────────────│
 │                    │ ◄────────────────────── │ SignedTx                │
 │ ◄──────────────────                          │                         │
 │ SignedTx                                                                │
```

---

## 8. Message signing

The same shape on every chain — the chain dispatch picks the right algorithm.

```
App                Binding                   jova-core              jova-core-chains
 │                    │                         │                         │
 │ wallet.signMessage(msg: .evmTypedDataV4(json))   │                         │
 │ ───────────────────►                          │                         │
 │                    │ ffi: jova_wallet_sign_message(handle, msg: SignableMessage)   │
 │                    │ ─────────────────────────►                         │
 │                    │                         │ derive XPrv             │
 │                    │                         │                         │
 │                    │                         │ chains::evm::eip712      │
 │                    │                         │ ───────────────────────►│
 │                    │                         │  parse typed data       │
 │                    │                         │  hashStruct(domain) +   │
 │                    │                         │  hashStruct(message)    │
 │                    │                         │  digest = keccak256(    │
 │                    │                         │    "\x19\x01" || dh || mh)│
 │                    │                         │  secp256k1.sign(digest) │
 │                    │                         │  encode rsv             │
 │                    │                         │ ◄───────────────────────│
 │                    │ ◄────────────────────── │ Signature{ hex }        │
 │ ◄──────────────────                          │                         │
 │ Signature                                                               │
```

**Notes per scheme**

- `evmPersonalSign`: prefix `\x19Ethereum Signed Message:\n<len>` then keccak256, then sign.
- `evmTypedDataV4`: per EIP-712, hashed-struct over the typed JSON.
- `solana`: raw ed25519 over message bytes.
- `bitcoin` (BIP-322): full BIP-322 simple signature over a virtual tx; legacy variant uses Bitcoin Core's `signMessage` prefix scheme.

---

## 9. Address validation

```
App           Binding             jova-core              jova-core-chains
 │              │                    │                         │
 │ JovaWallet.isValidAddress("bc1q...", on: .bitcoin)          │
 │ ─────────────►                    │                         │
 │              │ ffi call           │                         │
 │              │ ──────────────────►│                         │
 │              │                    │ chains::btc::validate   │
 │              │                    │ ───────────────────────►│
 │              │                    │  bech32 decode + checks │
 │              │                    │ ◄───────────────────────│
 │              │ ◄────────────────  │ true / false            │
 │ ◄────────────                     │                         │
 │ Bool                                                        │
```

**Notes**

- This is a static function — no `JovaWallet` instance required.
- Validation is purely syntactic: address format, checksum, network prefix. It does not check that an address has been used on-chain or has a balance.

---

## 10. Disposal

```
App                  Binding                  jova-core            jova-core-primitives
 │                      │                        │                        │
 │ wallet ⇒ deinit      │                        │                        │
 │ (Swift) / .close()   │                        │                        │
 │ ────────────────────►│                        │                        │
 │                      │ ffi: jova_wallet_drop(handle)                   │
 │                      │ ──────────────────────►│                        │
 │                      │                        │ Box::from_raw(handle)  │
 │                      │                        │ Drop → Zeroizing::drop │
 │                      │                        │ ──────────────────────►│
 │                      │                        │ memset(0, sizeof seed) │
 │                      │                        │ ◄──────────────────────│
 │                      │ ◄────────────────────  │                        │
 │ (handle nil)         │                        │                        │
```

**Notes**

- This is the only path where Rust memory holding the seed is freed. There is no double-free path (`Box::from_raw` is the unique owner).
- On Kotlin, `AutoCloseable.close()` triggers the same FFI call. On JavaScript, an explicit `wallet.destroy()` call is required (the WASM runtime has no destructor mechanism).
- See `memory-and-keys.md` for the full secret-clearing contract.

---

## Cross-cutting: error propagation

Every flow above can fail. Errors propagate as follows:

```
Rust source → JovaError variant
            → uniffi-rs marshals to Swift / Kotlin error type
            → wasm-bindgen marshals to JS Error subclass
            → consumer sees idiomatic exception / Result
```

`error-model.md` documents every variant and its per-binding mapping.

---

## Cross-cutting: where the seed lives

At any moment during a signing flow, the seed and any derived `XPrv` are held in Rust-owned memory inside `Zeroizing<>` wrappers. They never cross FFI. The only secret material that ever leaves Rust is:

- The mnemonic *string*, when the app generates a fresh one (it has to — the user needs to see the words).
- The signature (which is not secret — it's the output).

Public keys and addresses cross FFI freely (they are public).

This invariant is what `memory-and-keys.md` codifies and what `security.md` audits.
