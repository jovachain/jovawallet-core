# Chain Registry

Authoritative reference for what each `JovaChain` value means: derivation path, address format, signing input, signing output, and which Rust crate handles it.

## Quick reference

| `JovaChain` | Curve | Derivation path | Address format | Tx family | Crate |
|---|---|---|---|---|---|
| `ethereum` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559 (type 2), chainId=1 | `alloy` |
| `polygon` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=137 | `alloy` |
| `bsc` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=56 | `alloy` |
| `arbitrum` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=42161 | `alloy` |
| `optimism` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=10 | `alloy` |
| `base` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=8453 | `alloy` |
| `bitcoin` | secp256k1 | `m/84'/0'/0'/0/0` | bech32 P2WPKH `bc1q…` | BIP-174 PSBT | `bdk_wallet` |
| `solana` | ed25519 | `m/44'/501'/0'/0'` | base58 (32 B pubkey) | v0 versioned tx | Anza split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`) |
| `xrp` | secp256k1 | `m/44'/144'/0'/0/0` | base58check `r…` | canonical XRPL tx | `xrpl-rust` |
| `customEvm(N)` | secp256k1 | `m/44'/60'/0'/0/0` | EIP-55 `0x…` | EIP-1559, chainId=N | `alloy` |

> **All EVM chains share the same derivation path and the same address.** The chain difference appears only in the signed transaction's `chainId`. This matches every other multi-EVM wallet on the market and is intentional.

---

## Per-chain detail

### EVM family (Ethereum, Polygon, BSC, Arbitrum, Optimism, Base, custom)

| | |
|---|---|
| **Curve** | secp256k1 |
| **HD path** | `m/44'/60'/0'/0/<account>` (BIP-44, coin type 60) |
| **Address derivation** | `keccak256(uncompressed_pubkey[1:])[12:]`, EIP-55 checksummed |
| **Tx encoding** | EIP-1559 type-2 (`0x02 || rlp(...)`). No legacy (type-0) signing in v1; if backend ever sends a legacy unsigned tx, it's rejected as `malformedUnsignedTx("legacy_tx_not_supported")`. |
| **Tx signing flow** | Decode `UnsignedTx.evm` → build `alloy::consensus::TxEip1559` → sign hash with secp256k1 → serialize signed tx → return `0x`-prefixed hex + `keccak256` tx hash. |
| **Message signing** | EIP-191 (`personal_sign` prefix) and EIP-712 v4 (`signTypedData_v4`). No `eth_sign` (raw hash) — refused by SDK as unsafe. |
| **chainId source** | `UnsignedTx.evm.chainId` directly, or for `JovaChain.customEvm(N)` from the variant payload — the two must match if both are present. |
| **Access list** | EIP-2930 access lists supported in `UnsignedTx.evm.accessList`. Optional. |
| **Gas units** | All gas/fee fields are decimal strings to avoid `UInt64` overflow on chains with extreme fees. |

### Bitcoin

| | |
|---|---|
| **Curve** | secp256k1 |
| **Standard** | BIP-84 (native SegWit, P2WPKH, `bc1q…`) |
| **HD path** | `m/84'/0'/0'/0/<account>` |
| **Address derivation** | `bech32(hrp="bc", witness_version=0, witness_program=hash160(compressed_pubkey))` |
| **Tx signing flow** | Decode base64 PSBT → identify inputs the wallet can sign → sign all signable inputs with the appropriate derived key → finalize witnesses → serialize the resulting tx → return hex + `sha256d` tx hash. |
| **What's not signed** | Inputs whose script the wallet has no key for are returned in the PSBT as-signed-by-others (a multi-party PSBT flow); the SDK does not finalize a partially-signed tx in that case — backend or app coordinates the next signer. |
| **Message signing** | BIP-322 (preferred for new code), legacy `signMessage` (for compatibility with services that haven't moved). Specified by the `BtcMsgScheme` field in `SignableMessage.bitcoin`. |
| **Why BIP-84 not BIP-44** | BIP-44 P2PKH (`1…`) is legacy. Modern wallets default to native SegWit. The legacy iOS BIP-44 stub never had a signing path and held no funds — unification cost is zero. ADR D4. |
| **Taproot (BIP-86)** | Not at v1. Will be added as a separate `JovaChain.bitcoinTaproot` when backend PSBT-v2 construction stabilizes. BDK supports it fully. |

### Solana

| | |
|---|---|
| **Curve** | ed25519 |
| **Standard** | BIP-44 with hardened-only path (Phantom convention; SLIP-10 derivation) |
| **HD path** | `m/44'/501'/<account>'/0'` |
| **Address derivation** | The 32-byte ed25519 public key, base58-encoded. (Solana addresses *are* ed25519 public keys — no extra hashing.) |
| **Tx signing flow** | Decode base64 message → reconstruct `VersionedTransaction` v0 with the supplied `recentBlockhash` → sign the message bytes with the wallet's ed25519 key → serialize → return wire-format hex + signature. |
| **Address Lookup Tables (ALTs)** | The SDK respects ALTs in v0 messages (the backend resolves them when constructing the message; the SDK signs whatever the backend produced). |
| **Message signing** | Plain ed25519 over the message bytes; signature is base58-encoded. |

### XRP

| | |
|---|---|
| **Curve** | secp256k1 (matching XRPL's default; ed25519 is also XRPL-supported but not v1) |
| **HD path** | `m/44'/144'/0'/0/<account>` |
| **Address derivation** | `base58check(token=0x00 + ripemd160(sha256(compressed_pubkey)))`, prefix `r`. |
| **Tx signing flow** | Parse canonical JSON → serialize per XRPL canonical binary form → sign the serialized bytes with secp256k1 → re-serialize with `TxnSignature` field → return hex + `sha512_half` tx hash. |
| **Destination tags** | Encoded as `DestinationTag` field in tx JSON. SDK does not validate them — passes through whatever the backend produced. |

### Custom Jova chain (when it ships)

If the Jova chain is EVM-equivalent at the tx level (which is the standard "EVM-compatible" definition):

- Use `JovaChain.customEvm(chainId: <jova-chain-id>)`.
- No SDK code change required.
- Add one vector triplet to `spec/test-vectors.json` (mnemonic → address → signed tx hex).
- Tag a minor release.

If the Jova chain is non-EVM with novel cryptography or a novel transaction format:

- Add a new variant to `JovaChain`.
- Add a new variant to `UnsignedTx`.
- Add a `chains::jova` module under `crates/jova-core-chains/src/`.
- Implement `ChainSigner`.
- Add three vectors.
- Tag a minor release. Apps don't see any difference until they want to use the new variant.

The boundary is clean: apps that don't use the new chain are unaffected.

---

## Adding a new chain — checklist

1. Confirm a production-quality Rust crate exists. (If not, raise the question — pulling in unaudited or low-quality crypto is the one thing this SDK refuses.)
2. Add a case to `JovaChain` in `crates/jova-core/src/chain.rs`.
3. Add a row to the table at the top of this file.
4. Add an `UnsignedTx` variant if the chain's transaction shape doesn't fit an existing one.
5. Implement `ChainSigner` for the chain in `crates/jova-core-chains/src/<chain>/`.
6. Wire it into the dispatch in `crates/jova-core-chains/src/lib.rs`.
7. Add at least **three vector triplets** to `spec/test-vectors.json`:
   - mnemonic → derived address
   - mnemonic + unsigned tx 1 → signed tx hex
   - mnemonic + unsigned tx 2 (different scenario) → signed tx hex
8. Add to every binding's surface tests (`ApiSurfaceTests.swift`, `ApiSurfaceTest.kt`, `api-surface.test.ts`).
9. Update `docs/chains.md`, `docs/api.md` (if `UnsignedTx` grew), and `docs/integration-*.md` (if integration story is non-obvious).
10. CI must be green on every binding before the PR merges.

> Step 7 is non-negotiable. A chain without vectors is not "supported" — it's untested code that will silently rot.

---

## Coverage on hardware-wallet firmware

Of the chain-specific crates, only `jova-core-primitives` is `no_std`-clean. `bdk_wallet`, `alloy`, the Solana split crates, and `xrpl-rust` are `std`-using.

Hardware-wallet firmware that wants to support a chain must either:

- Use `jova-core-primitives` for derivation and raw-curve signing only, and implement the chain-specific encoding in firmware-side code (this is what current Bitcoin hardware wallets do — they sign serialized tx bytes, not parsed PSBT internals).
- Or pull in the lower-level `rust-bitcoin` / equivalent `no_std`-friendly subset directly.

`integration-hardware.md` covers this in detail.
