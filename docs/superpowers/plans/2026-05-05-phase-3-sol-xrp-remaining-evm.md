# Phase 3: Solana + XRP + Remaining EVM Chains

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every remaining v1 chain shipping with full vector parity. Polygon, BSC, Arbitrum, Optimism, Base via the existing EVM signer (just enum entries + vectors). XRP via `xrpl` crate. Solana via Anza's split crates with SLIP-10 derivation. All vector-parity-tested across Rust + Swift + Kotlin. Tag `v0.5.0`.

**Architecture:** Three independent sub-phases that can run in parallel with separate sub-agents. None individually carries BTC's risk profile. After this phase, every chain in `JovaChain` is fully implemented; v0.5.0 is the SDK that the iOS and Android apps consume in Phase 4.

**Tech Stack:** As Phase 2, plus `xrpl` and the Anza Solana split crates (`solana-keypair`, `solana-pubkey`, `solana-signature`, `solana-transaction`, `solana-message`) — all declared in Phase 0's `[workspace.dependencies]`. SLIP-10 derivation requires the `slip-10` crate (also in workspace deps).

**Preconditions:**
- Phase 2 complete; `v0.2.0` tagged.
- `ChainSigner` trait stable.
- `JovaError` reason vocabulary frozen at v0.2.0 in `spec/errors.md`.
- Anza Solana split crates and `xrpl` confirmed compiling natively (and ideally on WASM) per Phase -1's feasibility report.

**Exit criteria:**
- All vectors green on Rust + Swift + Kotlin for every v1 chain.
- Differential XRP test against `xrpl-py` passes on 100+ random tx shapes.
- Solana versioned-tx tests cover both legacy-message-in-v0 and ALT-using cases.
- WASM compile-smoke remains green.
- Tag `v0.5.0` on `main`.

---

## Sub-phase 3a — Other EVM chains (~2 days)

**Goal:** Polygon, BSC, Arbitrum, Optimism, Base all signing through the existing `EvmSigner`. Same code path; only `chainId` changes; vectors confirm byte-identical output.

### Task 3a.1: Confirm `JovaChain` and `chain_label_from_evm_chain_id` cover every chain

**Files:**
- Verify: `crates/jova-core/src/chain.rs`
- Verify: `crates/jova-core/src/wallet.rs`

- [ ] **Step 1: Verify enum coverage**

```bash
grep -A12 "pub enum JovaChain" crates/jova-core/src/chain.rs
```

Expected: every variant present — `Ethereum`, `Polygon`, `Bsc`, `Arbitrum`, `Optimism`, `Base`, `CustomEvm`, `Bitcoin`, `Solana`, `Xrp`. (The non-EVM ones are placeholders; SOL/XRP get implemented in 3b/3c.)

- [ ] **Step 2: Verify `chain_label_from_evm_chain_id` covers every EVM chainId**

```bash
grep -A12 "fn chain_label_from_evm_chain_id" crates/jova-core/src/wallet.rs
```

Expected entries: `1` → `ethereum`, `137` → `polygon`, `56` → `bsc`, `42161` → `arbitrum`, `10` → `optimism`, `8453` → `base`, `_` → `customEvm`. If any is missing, add it.

- [ ] **Step 3: Smoke build**

```bash
cargo build --workspace
```

### Task 3a.2: Author 7 vectors for the new EVM chains

**Files:**
- Modify: `spec/test-vectors.json`
- Modify: `tools/vector-capture/inputs.json`
- Modify: `tools/vector-capture/capture.sh`

- [ ] **Step 1: Add inputs**

The address is identical across all EVM chains for a given seed (BIP-44 coin type 60 is shared). One *address* vector per chain × 1 mnemonic = 5 new address vectors confirming this. Plus 2 sign_tx vectors at typical mainnet gas prices for Polygon and Arbitrum (the other 3 EVMs are functionally proven by signature-shape parity).

Add to `tools/vector-capture/inputs.json`:

```json
{
  "evm_other_chain_address": [
    { "id": "evm.address.polygon_account0_abandon", "chain": { "kind": "polygon" }, "mnemonic": "abandon" },
    { "id": "evm.address.bsc_account0_abandon",     "chain": { "kind": "bsc" },     "mnemonic": "abandon" },
    { "id": "evm.address.arbitrum_account0_abandon","chain": { "kind": "arbitrum" },"mnemonic": "abandon" },
    { "id": "evm.address.optimism_account0_abandon","chain": { "kind": "optimism" },"mnemonic": "abandon" },
    { "id": "evm.address.base_account0_abandon",    "chain": { "kind": "base" },    "mnemonic": "abandon" }
  ],
  "evm_other_chain_tx": [
    { "id": "evm.tx.polygon_transfer",  "chain": { "kind": "polygon" },  "tx_template": "polygon_transfer" },
    { "id": "evm.tx.arbitrum_transfer", "chain": { "kind": "arbitrum" }, "tx_template": "arbitrum_transfer" }
  ]
}
```

- [ ] **Step 2: Capture reference values**

Extend `tools/vector-capture/capture.sh` to spawn `anvil` per chainId and run `cast wallet sign-tx` against each, capturing `signed_hex` and `tx_hash`.

```bash
./tools/vector-capture/capture.sh evm_other_chain
# Outputs spec/test-vectors.json fragments for the 7 new vectors.
```

- [ ] **Step 3: Append to spec/test-vectors.json**

Bump `version` to `"0.4"`. Run `cargo run -p jova-verify-spec` — expected `OK`.

### Task 3a.3: Vector tests pass on every binding

- [ ] **Step 1: The Phase 1 EVM vector test files iterate over every `address` and `sign_tx` vector whose chain kind is in the EVM family. The new vectors are picked up automatically. Run:**

```bash
just test
just build-ios && (cd bindings/swift && swift test)
just build-android && (cd bindings/kotlin && ./gradlew :jova-core:test)
```

Expected: 7 new EVM vectors pass on every binding.

- [ ] **Step 2: Commit**

```bash
git add spec/test-vectors.json tools/vector-capture/
git commit -m "feat(spec): 7 vectors covering Polygon, BSC, Arbitrum, Optimism, Base"
```

### Task 3a.4: Tag v0.3.0 (optional intermediate tag)

```bash
git tag -a v0.3.0 -m "v0.3.0 — Phase 3a (full EVM family)"
git push origin v0.3.0
```

(Optional — the team may prefer to roll Phase 3a into the v0.5.0 tag if all three sub-phases land together. If the sub-phases run in parallel and 3a finishes first, tagging now makes app teams' beta-testing easier.)

---

## Sub-phase 3b — XRP (~5–7 days)

**Goal:** XRP address derivation, canonical XRPL serialization, secp256k1 signing, BIP-44 coin type 144. Vector parity + differential test against `xrpl-py`.

### Task 3b.1: BIP-44 path helper for XRP

**Files:**
- Modify: `crates/jova-core-primitives/src/path.rs`
- Modify: `crates/jova-core-primitives/tests/bip84.rs` (rename or add)

- [ ] **Step 1: Add a `bip44_path` helper if not already present**

Check `crates/jova-core-primitives/src/path.rs`. If `bip44_path(coin_type, account, change, index)` doesn't exist, add it — same shape as `bip84_path` but with the leading purpose `44` instead of `84`:

```rust
impl DerivationPath {
    pub fn bip44_path(coin_type: u32, account: u32, change: u32, index: u32) -> Result<Self, PathError> {
        if coin_type >= HARDENED_OFFSET || account >= HARDENED_OFFSET || change >= HARDENED_OFFSET || index >= HARDENED_OFFSET {
            return Err(PathError::IndexOutOfRange);
        }
        Ok(Self {
            indices: alloc::vec![
                44 + HARDENED_OFFSET,
                coin_type + HARDENED_OFFSET,
                account + HARDENED_OFFSET,
                change,
                index,
            ],
        })
    }
}
```

- [ ] **Step 2: Test**

```rust
#[test]
fn bip44_xrp_path() {
    let p = DerivationPath::bip44_path(144, 0, 0, 0).unwrap();
    let parsed = DerivationPath::parse("m/44'/144'/0'/0/0").unwrap();
    assert_eq!(p, parsed);
}
```

Add to `crates/jova-core-primitives/tests/bip84.rs` (or rename the file to `paths.rs` since it now covers both BIP-84 and BIP-44).

- [ ] **Step 3: Commit**

```bash
git add crates/jova-core-primitives/
git commit -m "feat(primitives): bip44_path helper (used by XRP and EVM)"
```

### Task 3b.2: XRP address derivation

**Files:**
- Create: `crates/jova-core-chains/src/xrp/mod.rs`
- Create: `crates/jova-core-chains/src/xrp/address.rs`
- Modify: `crates/jova-core-chains/src/lib.rs`
- Create: `crates/jova-core-chains/tests/xrp_address.rs`

- [ ] **Step 1: Failing test**

`crates/jova-core-chains/tests/xrp_address.rs`:

```rust
use jova_core_chains::xrp::{derive_xrp_address, validate_xrp_address};
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn xrp_address_for_abandon_seed_account_0() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/44'/144'/0'/0/0").unwrap();
    let xprv = derive_secp256k1(&seed, &path).unwrap();
    let addr = derive_xrp_address(&xprv);
    // Expected captured from XRPL reference signer (xrpl-cli) against this seed:
    let expected = include_str!("../../../tools/xrp-vector-capture/captures/abandon_account0.address");
    assert_eq!(addr, expected.trim());
}

#[test]
fn validates_known_xrp_addresses() {
    // From XRPL test fixtures.
    assert!(validate_xrp_address("rEAaa1Lv5UfL1Z4xT7yT1n2tqxJfHXn1Hh"));
    assert!(validate_xrp_address("rJrRMgiRgrU6hPFydSubVjf3vLPxYbYZf"));
}

#[test]
fn rejects_malformed_xrp_addresses() {
    assert!(!validate_xrp_address("rEAaa1Lv5UfL1Z4xT7yT1n2tqxJfHXn1Hg"));   // bad checksum
    assert!(!validate_xrp_address("xxx"));
    assert!(!validate_xrp_address("0x0000000000000000000000000000000000000000"));
}
```

- [ ] **Step 2: Capture XRP reference values**

`tools/xrp-vector-capture/capture.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
mkdir -p tools/xrp-vector-capture/captures
MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Use xrpl-cli (or python xrpl-py) to derive the address from the same seed.
ADDR=$(python3 -c "
from xrpl.wallet import Wallet
from mnemonic import Mnemonic
seed = Mnemonic('english').to_seed('$MNEMONIC')
w = Wallet.from_seed(seed.hex()[:32], algorithm='secp256k1', master_address=None)
print(w.classic_address)
")
echo "$ADDR" > tools/xrp-vector-capture/captures/abandon_account0.address
```

(Note: the seed-to-keypair derivation pattern in xrpl-py expects an XRPL "secret" not a BIP-39 seed — adjust the script. The agent verifies the cross-reference produces a stable address; if xrpl-py's model differs, capture from `rippled`'s `wallet_propose` RPC against the bip39 master seed via a small Python helper.)

```bash
chmod +x tools/xrp-vector-capture/capture.sh
./tools/xrp-vector-capture/capture.sh
cat tools/xrp-vector-capture/captures/abandon_account0.address
# Should print an `r…` address
```

- [ ] **Step 3: Run; confirm fails**

```bash
cargo test -p jova-core-chains --test xrp_address
```

Expected: `unresolved import jova_core_chains::xrp`.

- [ ] **Step 4: Implement**

`crates/jova-core-chains/src/xrp/address.rs`:

```rust
use jova_core_primitives::XPrv;
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

const XRPL_ADDRESS_VERSION: u8 = 0x00;
const XRPL_BASE58_ALPHABET: &[u8] = b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";

/// Derive an XRP classic address (`r…`) from an XPrv.
pub fn derive_xrp_address(xprv: &XPrv) -> String {
    let pubkey = xprv.public_key_compressed();
    let mut sha = Sha256::new();
    sha.update(pubkey);
    let sha_digest = sha.finalize();
    let mut rip = Ripemd160::new();
    rip.update(sha_digest);
    let rip_digest = rip.finalize();

    // Prepend version, append 4-byte checksum (sha256d truncated).
    let mut payload = Vec::with_capacity(1 + 20);
    payload.push(XRPL_ADDRESS_VERSION);
    payload.extend_from_slice(&rip_digest);
    let checksum = sha256d(&payload);
    payload.extend_from_slice(&checksum[..4]);

    base58_encode_xrpl(&payload)
}

pub fn validate_xrp_address(s: &str) -> bool {
    let bytes = match base58_decode_xrpl(s) {
        Some(b) => b,
        None => return false,
    };
    if bytes.len() != 25 { return false; }
    if bytes[0] != XRPL_ADDRESS_VERSION { return false; }
    let payload = &bytes[..21];
    let provided_checksum = &bytes[21..];
    let expected = sha256d(payload);
    &expected[..4] == provided_checksum
}

fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    Sha256::digest(&first).into()
}

fn base58_encode_xrpl(input: &[u8]) -> String {
    // Standard base58 with the XRPL alphabet.
    let mut x = num_bigint_dig::BigUint::from_bytes_be(input);
    let radix = num_bigint_dig::BigUint::from(58u32);
    let zero = num_bigint_dig::BigUint::from(0u32);
    let mut out = Vec::new();
    while x > zero {
        let r = (&x % &radix).to_bytes_be();
        let idx = r.first().copied().unwrap_or(0) as usize;
        out.push(XRPL_ADDRESS_VERSION); // placeholder; we'll overwrite below
        let last = out.last_mut().unwrap();
        *last = XRPL_BASE58_ALPHABET[idx];
        x /= &radix;
    }
    // Leading-zero handling: each leading 0x00 byte becomes one `r` (the alphabet's index 0).
    for &b in input.iter() {
        if b != 0 { break; }
        out.push(XRPL_BASE58_ALPHABET[0]);
    }
    out.reverse();
    String::from_utf8(out).expect("XRPL alphabet is ASCII")
}

fn base58_decode_xrpl(s: &str) -> Option<Vec<u8>> {
    let mut x = num_bigint_dig::BigUint::from(0u32);
    let radix = num_bigint_dig::BigUint::from(58u32);
    for ch in s.bytes() {
        let idx = XRPL_BASE58_ALPHABET.iter().position(|&c| c == ch)?;
        x = x * &radix + num_bigint_dig::BigUint::from(idx as u32);
    }
    let mut bytes = x.to_bytes_be();
    // Restore leading zero bytes.
    let zero_count = s.bytes().take_while(|&b| b == XRPL_BASE58_ALPHABET[0]).count();
    let mut padded = vec![0u8; zero_count];
    padded.append(&mut bytes);
    Some(padded)
}
```

(The `xrpl` crate exposes `xrpl::core::keypairs::derive_classic_address` and base58 helpers; using the crate's helpers is preferred over hand-rolling. If the crate exposes them, replace the hand-rolled `base58_encode_xrpl` / `base58_decode_xrpl` with `xrpl::core::addresscodec::*` functions. The hand-rolled version above is a fallback for the case where the agent finds the crate's helpers aren't accessible.)

`crates/jova-core-chains/src/xrp/mod.rs`:

```rust
pub mod address;
pub use address::{derive_xrp_address, validate_xrp_address};
```

Add to `crates/jova-core-chains/src/lib.rs`:

```rust
pub mod xrp;
```

Add to `crates/jova-core-chains/Cargo.toml`:

```toml
xrpl.workspace = true
ripemd.workspace = true
num-bigint-dig = "0.8"
```

- [ ] **Step 5: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test xrp_address
```

- [ ] **Step 6: Commit**

```bash
git add crates/jova-core-chains/ tools/xrp-vector-capture/
git commit -m "feat(chains/xrp): address derivation + validation"
```

### Task 3b.3: XRP transaction signing

**Files:**
- Create: `crates/jova-core-chains/src/xrp/tx.rs`
- Modify: `crates/jova-core-chains/src/xrp/mod.rs`
- Create: `crates/jova-core-chains/tests/xrp_tx.rs`
- Modify: `crates/jova-core-chains/src/unsigned_tx.rs` (Xrp variant already exists from Phase 1 typed FFI; verify)

- [ ] **Step 1: Capture reference values**

`tools/xrp-vector-capture/capture-tx.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
ADDR=$(cat tools/xrp-vector-capture/captures/abandon_account0.address)

# Construct a Payment tx with destination tag.
TX_JSON=$(python3 -c "
import json
tx = {
    'TransactionType': 'Payment',
    'Account': '$ADDR',
    'Destination': 'rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe',
    'DestinationTag': 12345,
    'Amount': '1000000',
    'Fee': '12',
    'Sequence': 1,
    'Flags': 0,
}
print(json.dumps(tx))
")
echo "TX_JSON=$TX_JSON" > tools/xrp-vector-capture/captures/payment_dt.tx_json
echo "$TX_JSON" >> tools/xrp-vector-capture/captures/payment_dt.tx_json

# Sign via xrpl-py reference.
SIGNED=$(python3 -c "
import json
from xrpl.wallet import Wallet
from xrpl.core.binarycodec import encode, encode_for_signing
from xrpl.transaction import sign

# Wallet from same BIP-39 seed (hex-encoded master seed via xrpl).
# Adjust to the actual xrpl-py API for BIP-44 derivation.
w = Wallet(seed='sn3nxiW7v8KXzPzAqzyHXbSSKNuN9', algorithm='secp256k1')   # xrpl-cli output
tx_dict = json.loads('$TX_JSON')
tx_dict['Account'] = w.classic_address
signed = sign(tx_dict, w)
print(json.dumps({'signed_hex': encode(signed.to_dict()), 'tx_hash': signed.get_hash()}))
")
echo "$SIGNED" > tools/xrp-vector-capture/captures/payment_dt.signed
```

(The xrpl-py seed format isn't BIP-39 directly; an adapter step is needed to convert. The agent uses `rippled`'s `wallet_propose` RPC against the bip39 master seed *or* uses `xrpl-cli`'s account-from-secret path. Document the exact bridging script in `tools/xrp-vector-capture/README.md`.)

- [ ] **Step 2: Failing test**

```rust
use jova_core_chains::xrp::{sign_xrp_tx};
// ...

#[test]
fn signs_payment_with_destination_tag() {
    let xprv = test_xprv();
    let tx_json = include_str!("../../../tools/xrp-vector-capture/captures/payment_dt.tx_json");
    let signed = sign_xrp_tx(&xprv, tx_json).unwrap();
    let captured: serde_json::Value = serde_json::from_str(
        include_str!("../../../tools/xrp-vector-capture/captures/payment_dt.signed")
    ).unwrap();
    let expected_hex = captured["signed_hex"].as_str().unwrap();
    let expected_hash = captured["tx_hash"].as_str().unwrap();
    assert_eq!(signed.0.to_uppercase(), expected_hex.to_uppercase());
    assert_eq!(signed.1.to_uppercase(), expected_hash.to_uppercase());
}
```

- [ ] **Step 3: Implement signing using `xrpl` crate**

`crates/jova-core-chains/src/xrp/tx.rs`:

```rust
use jova_core_primitives::XPrv;
use crate::error::ChainError;
use serde_json::Value;
use xrpl::core::binarycodec::{encode, encode_for_signing};
use secp256k1::{Secp256k1, SecretKey, Message};
use sha2::{Digest, Sha512};

/// Sign a canonical XRPL transaction JSON. Returns (signed_hex, tx_hash) where
/// signed_hex is the binary-encoded signed transaction and tx_hash is its
/// SHA512Half ID.
pub fn sign_xrp_tx(xprv: &XPrv, tx_json: &str) -> Result<(String, String), ChainError> {
    // Parse JSON.
    let mut tx: Value = serde_json::from_str(tx_json)
        .map_err(|_| ChainError::MalformedUnsignedTx("xrp_invalid_json".into()))?;

    // Required field check.
    for required in &["TransactionType", "Account"] {
        if tx.get(required).is_none() {
            return Err(ChainError::MalformedUnsignedTx(
                format!("xrp_missing_required_field:{}", required)
            ));
        }
    }

    // Inject SigningPubKey (compressed pubkey, hex uppercase).
    let pubkey_compressed = xprv.public_key_compressed();
    tx["SigningPubKey"] = Value::String(hex::encode_upper(pubkey_compressed));

    // Canonical serialization for signing.
    let signing_bytes = encode_for_signing(&tx)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_serialize_failed:{:?}", e)))?;
    let mut hasher = Sha512::new();
    hasher.update(&signing_bytes);
    let digest = hasher.finalize();
    let half = &digest[..32];

    // Sign with secp256k1 (XRPL uses DER encoding).
    let secp = Secp256k1::signing_only();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("secp256k1_invalid_secret".into()))?;
    let msg = Message::from_digest_slice(half)
        .map_err(|_| ChainError::SigningFailed("xrp_digest_invalid".into()))?;
    let sig = secp.sign_ecdsa(&msg, &sk);
    let der = sig.serialize_der();

    tx["TxnSignature"] = Value::String(hex::encode_upper(der));

    // Re-encode with TxnSignature included.
    let final_bytes = encode(&tx)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_serialize_failed:{:?}", e)))?;
    let final_hex = hex::encode_upper(&final_bytes);

    // Compute tx hash (SHA512Half over the prefixed signed bytes; XRPL spec).
    let mut h = Sha512::new();
    h.update(b"\x54\x58\x4E\x00");   // "TXN\0"
    h.update(&final_bytes);
    let h_digest = h.finalize();
    let tx_hash = hex::encode_upper(&h_digest[..32]);

    Ok((final_hex, tx_hash))
}
```

`crates/jova-core-chains/src/xrp/mod.rs`:

```rust
pub mod address;
pub mod tx;

pub use address::{derive_xrp_address, validate_xrp_address};
pub use tx::sign_xrp_tx;
```

- [ ] **Step 4: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test xrp_tx
```

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-chains/ tools/xrp-vector-capture/
git commit -m "feat(chains/xrp): canonical XRPL signing"
```

### Task 3b.4: XrpSigner trait + JovaWallet dispatch

**Files:**
- Modify: `crates/jova-core-chains/src/xrp/mod.rs`
- Modify: `crates/jova-core/src/wallet.rs`
- Modify: `crates/jova-core/src/chain.rs`

- [ ] **Step 1: ChainSigner impl**

Append to `crates/jova-core-chains/src/xrp/mod.rs`:

```rust
use crate::address::{Address as JovaAddress, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;
use jova_core_primitives::XPrv;

pub struct XrpSigner;

impl ChainSigner for XrpSigner {
    fn derive_address(&self, key: &XPrv) -> Result<JovaAddress, ChainError> {
        Ok(JovaAddress {
            chain: "xrp".to_string(),
            value: derive_xrp_address(key),
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        validate_xrp_address(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Xrp { tx_json } => {
                let (raw_hex, tx_hash) = sign_xrp_tx(key, tx_json)?;
                Ok(SignedTx { chain: "xrp".to_string(), raw_hex, tx_hash })
            }
            _ => Err(ChainError::MalformedUnsignedTx("expected_xrp_variant".into())),
        }
    }

    fn sign_message(&self, _key: &XPrv, _msg: &SignableMessage) -> Result<Signature, ChainError> {
        // XRP message signing isn't a standard XRPL feature; reject.
        Err(ChainError::MalformedSignableMessage("xrp_message_signing_unsupported".into()))
    }
}
```

- [ ] **Step 2: Wire into JovaWallet**

Modify `crates/jova-core/src/wallet.rs`'s `sign_tx` and `address`:

```rust
// In sign_tx:
UnsignedTx::Xrp { .. } => {
    let xprv = self.derive_path("m/44'/144'/0'/0/0")?;
    Ok(jova_core_chains::xrp::XrpSigner.sign_tx(&xprv, unsigned)?)
}

// In address:
JovaChain::Xrp => {
    let xprv = self.derive_path("m/44'/144'/0'/0/0")?;
    Ok(jova_core_chains::xrp::XrpSigner.derive_address(&xprv)?)
}
```

Update the JovaChain `derivation_path` and `label`:

```rust
Self::Xrp => "m/44'/144'/0'/0/0",   // in derivation_path
Self::Xrp => "xrp",                  // in label
```

- [ ] **Step 3: Build & smoke**

```bash
cargo build --workspace
cargo test --workspace
```

- [ ] **Step 4: Commit**

```bash
git add crates/jova-core-chains/src/xrp/ crates/jova-core/src/
git commit -m "feat(core): XRP dispatch in JovaWallet"
```

### Task 3b.5: Differential test against `xrpl-py`

**Files:**
- Create: `tools/xrp-diff/run.py`
- Create: `tools/xrp-diff/Cargo.toml` (Rust harness)
- Create: `tools/xrp-diff/src/main.rs`

- [ ] **Step 1: Build the Python reference signer**

`tools/xrp-diff/run.py`:

```python
#!/usr/bin/env python3
"""
Generate 100 random XRPL Payment txs, sign each with both xrpl-py and the
SDK, assert byte-identical signed_hex and tx_hash.
"""
import json, random, secrets, subprocess, sys

from xrpl.wallet import Wallet
from xrpl.core.binarycodec import encode

SDK_BIN = "./target/release/jova-xrp-diff"   # built from tools/xrp-diff/

# Single shared mnemonic; each tx differs by Sequence, Amount, Destination.
MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
SDK_ADDR = subprocess.check_output([SDK_BIN, "address", MNEMONIC]).decode().strip()
PY_ADDR = ...   # convert from BIP-39 master seed via the bridging script

assert SDK_ADDR == PY_ADDR, f"address mismatch: SDK={SDK_ADDR} PY={PY_ADDR}"

mismatches = 0
for i in range(100):
    tx = {
        "TransactionType": "Payment",
        "Account": SDK_ADDR,
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "DestinationTag": random.randint(1, 1_000_000),
        "Amount": str(random.randint(100_000, 10_000_000)),
        "Fee": "12",
        "Sequence": i + 1,
        "Flags": 0,
    }

    # SDK signs.
    sdk_out = json.loads(subprocess.check_output([SDK_BIN, "sign", json.dumps(tx)]))

    # xrpl-py signs.
    w = Wallet(seed="...", algorithm="secp256k1")
    py_signed = sign(tx, w)
    py_hex = encode(py_signed.to_dict())
    py_hash = py_signed.get_hash()

    if sdk_out["signed_hex"].upper() != py_hex.upper() or sdk_out["tx_hash"].upper() != py_hash.upper():
        mismatches += 1
        print(f"MISMATCH at iter {i}: SDK={sdk_out['signed_hex'][:32]}… PY={py_hex[:32]}…")

print(f"differential: {100 - mismatches}/100 match")
sys.exit(0 if mismatches == 0 else 1)
```

- [ ] **Step 2: Build the Rust harness**

`tools/xrp-diff/Cargo.toml`:

```toml
[package]
name = "jova-xrp-diff"
version = "0.0.0"
edition.workspace = true
publish = false

[[bin]]
name = "jova-xrp-diff"
path = "src/main.rs"

[dependencies]
jova-core.workspace = true
serde_json.workspace = true
```

`tools/xrp-diff/src/main.rs`:

```rust
use jova_core::{JovaChain, JovaWallet, UnsignedTx};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args[1].as_str() {
        "address" => {
            let mnemonic = &args[2];
            let w = JovaWallet::from_mnemonic(mnemonic, "").unwrap();
            let a = w.address(&JovaChain::Xrp, 0).unwrap();
            println!("{}", a.value);
        }
        "sign" => {
            let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            let tx_json = &args[2];
            let w = JovaWallet::from_mnemonic(mnemonic, "").unwrap();
            let unsigned = UnsignedTx::Xrp { tx_json: tx_json.clone() };
            let signed = w.sign_tx(&unsigned).unwrap();
            println!("{}", serde_json::json!({
                "signed_hex": signed.raw_hex,
                "tx_hash": signed.tx_hash,
            }));
        }
        _ => panic!("unknown command"),
    }
}
```

- [ ] **Step 3: Run differential**

```bash
cargo build --release -p jova-xrp-diff
python3 tools/xrp-diff/run.py
```

Expected: `differential: 100/100 match`. If any mismatch, investigate before continuing.

- [ ] **Step 4: Commit**

```bash
git add tools/xrp-diff/
git commit -m "test(xrp): differential vs xrpl-py — 100/100 match"
```

### Task 3b.6: XRP vectors + parity tests

Same shape as Task 7/8 in Phase 2. 6 vectors total: 1 address, 2 sign_tx (Payment with destination tag, OfferCreate), 2 errors (`xrp_invalid_json`, `xrp_missing_required_field`), 1 negative validate-address.

After running the vector capture and parity tests, commit:

```bash
git add spec/test-vectors.json crates/jova-core/tests/vectors_xrp.rs bindings/swift/.../XrpVectorsTests.swift bindings/kotlin/.../XrpVectorsTest.kt
git commit -m "test: XRP vector parity Rust + Swift + Kotlin"
```

### Task 3b.7: Tag v0.4.0 (optional intermediate)

```bash
git tag -a v0.4.0 -m "v0.4.0 — Phase 3b XRP"
git push origin v0.4.0
```

---

## Sub-phase 3c — Solana (~7–10 days)

**Goal:** Solana address derivation (ed25519, base58), VersionedTransaction (v0) signing with ALT support, raw ed25519 message signing. Uses Anza's split crates and SLIP-10 derivation.

### Task 3c.1: SLIP-10 derivation in jova-core-primitives

**Files:**
- Create: `crates/jova-core-primitives/src/slip10.rs`
- Modify: `crates/jova-core-primitives/src/lib.rs`
- Create: `crates/jova-core-primitives/tests/slip10.rs`

- [ ] **Step 1: Failing test**

`crates/jova-core-primitives/tests/slip10.rs`:

```rust
use jova_core_primitives::{Mnemonic, DerivationPath, derive_ed25519};

const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn slip10_solana_path_derives_to_known_pubkey() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/44'/501'/0'/0'").unwrap();
    let xprv = derive_ed25519(&seed, &path).unwrap();
    let pubkey = xprv.public_key();
    // Expected captured from `solana-keygen pubkey` against this seed:
    let expected = include_str!("../../../tools/sol-vector-capture/captures/abandon_account0.pubkey_b58");
    assert_eq!(bs58::encode(pubkey).into_string(), expected.trim());
}
```

- [ ] **Step 2: Capture**

```bash
mkdir -p tools/sol-vector-capture/captures
solana-keygen recover -o /tmp/solkey.json 'prompt://?key=0/0&full-path=m/44%27/501%27/0%27/0%27' --force <<< "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
solana-keygen pubkey /tmp/solkey.json > tools/sol-vector-capture/captures/abandon_account0.pubkey_b58
rm /tmp/solkey.json
```

- [ ] **Step 3: Implement SLIP-10**

`crates/jova-core-primitives/src/slip10.rs`:

```rust
use crate::path::DerivationPath;
use crate::seed::Seed;
use slip_10::derive_key_from_path;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Ed25519 extended private key from SLIP-10 derivation.
/// NOT Clone — same secret-handling rules as XPrv.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Ed25519Xprv {
    pub(crate) secret_bytes: [u8; 32],
}

impl Ed25519Xprv {
    pub fn public_key(&self) -> [u8; 32] {
        use ed25519_dalek::SigningKey;
        let sk = SigningKey::from_bytes(&self.secret_bytes);
        sk.verifying_key().to_bytes()
    }

    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret_bytes
    }
}

impl core::fmt::Debug for Ed25519Xprv {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Ed25519Xprv(<redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveError {
    Slip10,
    HardenedRequired,
}

impl core::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Slip10 => f.write_str("SLIP-10 derivation failed"),
            Self::HardenedRequired => f.write_str("SLIP-10 ed25519 requires every component to be hardened"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeriveError {}

const HARDENED_OFFSET: u32 = 0x8000_0000;

pub fn derive_ed25519(seed: &Seed, path: &DerivationPath) -> Result<Ed25519Xprv, DeriveError> {
    // SLIP-10 ed25519 requires all path components to be hardened.
    for &i in &path.indices {
        if i < HARDENED_OFFSET { return Err(DeriveError::HardenedRequired); }
    }
    let key = derive_key_from_path(seed.as_bytes(), &path.indices)
        .map_err(|_| DeriveError::Slip10)?;
    Ok(Ed25519Xprv { secret_bytes: key.into() })
}
```

(Function names from `slip-10` 0.4 are approximate; if the API differs, the test in Step 1 is the contract — adjust the Rust code to match the crate.)

Add to `crates/jova-core-primitives/src/lib.rs`:

```rust
mod slip10;
pub use slip10::{derive_ed25519, Ed25519Xprv, DeriveError as Ed25519DeriveError};
```

Add `bs58 = "0.5"` to `crates/jova-core-primitives/Cargo.toml` for the test, or use a hand-rolled base58 (slip-10 outputs raw 32 bytes; the test base58-encodes for comparison with `solana-keygen pubkey`).

- [ ] **Step 4: Run; confirm passes**

```bash
cargo test -p jova-core-primitives --test slip10
```

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-primitives/ tools/sol-vector-capture/
git commit -m "feat(primitives): SLIP-10 ed25519 derivation"
```

### Task 3c.2: Solana address (base58 of pubkey)

**Files:**
- Create: `crates/jova-core-chains/src/sol/mod.rs`
- Create: `crates/jova-core-chains/src/sol/address.rs`
- Create: `crates/jova-core-chains/tests/sol_address.rs`

- [ ] **Step 1: Failing test**

`crates/jova-core-chains/tests/sol_address.rs`:

```rust
use jova_core_chains::sol::{derive_sol_address, validate_sol_address};
use jova_core_primitives::{Mnemonic, DerivationPath, derive_ed25519};

const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn sol_address_matches_solana_keygen() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/44'/501'/0'/0'").unwrap();
    let xprv = derive_ed25519(&seed, &path).unwrap();
    let addr = derive_sol_address(&xprv);
    let expected = include_str!("../../../tools/sol-vector-capture/captures/abandon_account0.pubkey_b58");
    assert_eq!(addr, expected.trim());
}

#[test]
fn validates_known_sol_addresses() {
    assert!(validate_sol_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
    assert!(validate_sol_address("So11111111111111111111111111111111111111112"));
}

#[test]
fn rejects_malformed_sol_addresses() {
    assert!(!validate_sol_address("not-base58!"));
    assert!(!validate_sol_address(""));
    assert!(!validate_sol_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWW"));   // too short
}
```

- [ ] **Step 2: Implement**

`crates/jova-core-chains/src/sol/address.rs`:

```rust
use jova_core_primitives::Ed25519Xprv;
use solana_pubkey::Pubkey;

pub fn derive_sol_address(xprv: &Ed25519Xprv) -> String {
    let pubkey_bytes = xprv.public_key();
    Pubkey::new_from_array(pubkey_bytes).to_string()
}

pub fn validate_sol_address(s: &str) -> bool {
    s.parse::<Pubkey>().is_ok() && s.len() >= 32 && s.len() <= 44
}
```

`crates/jova-core-chains/src/sol/mod.rs`:

```rust
pub mod address;
pub use address::{derive_sol_address, validate_sol_address};
```

Add to `crates/jova-core-chains/src/lib.rs`: `pub mod sol;`.

Add Anza split crates to `crates/jova-core-chains/Cargo.toml`:

```toml
solana-keypair.workspace     = true
solana-pubkey.workspace      = true
solana-signature.workspace   = true
solana-transaction.workspace = true
solana-message.workspace     = true
```

- [ ] **Step 3: Run; passes**

```bash
cargo test -p jova-core-chains --test sol_address
```

- [ ] **Step 4: Commit**

```bash
git add crates/jova-core-chains/
git commit -m "feat(chains/sol): address via solana-pubkey"
```

### Task 3c.3: Solana versioned transaction signing

**Files:**
- Create: `crates/jova-core-chains/src/sol/tx.rs`
- Modify: `crates/jova-core-chains/src/sol/mod.rs`
- Create: `crates/jova-core-chains/tests/sol_tx.rs`

- [ ] **Step 1: Capture reference values**

`tools/sol-vector-capture/capture-tx.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
# Use solana-cli sign-only to produce a reference v0 versioned transaction.
# Set up a local validator (solana-test-validator) for stable blockhash.
solana-test-validator --quiet &
TV_PID=$!
trap "kill $TV_PID" EXIT

solana config set --url http://localhost:8899
RECENT_BLOCKHASH=$(solana cluster-version | grep -oP '[A-Za-z0-9]{32,}' | head -1)
# Construct a SystemProgram::Transfer message base64 via solana-cli.
# Output: tools/sol-vector-capture/captures/transfer.message_base64
# Output: tools/sol-vector-capture/captures/transfer.signed_hex
# Output: tools/sol-vector-capture/captures/transfer.recent_blockhash
```

(Full bash + python helper to drive solana-cli's sign-only flow goes in the script.)

```bash
chmod +x tools/sol-vector-capture/capture-tx.sh
./tools/sol-vector-capture/capture-tx.sh
```

- [ ] **Step 2: Failing test**

```rust
use jova_core_chains::sol::sign_sol_tx;
// ...

#[test]
fn signs_v0_transfer_versioned_tx() {
    let xprv = test_xprv();
    let message_b64 = include_str!("../../../tools/sol-vector-capture/captures/transfer.message_base64");
    let blockhash = include_str!("../../../tools/sol-vector-capture/captures/transfer.recent_blockhash");
    let expected_hex = include_str!("../../../tools/sol-vector-capture/captures/transfer.signed_hex");

    let signed = sign_sol_tx(&xprv, message_b64.trim(), blockhash.trim()).unwrap();
    assert_eq!(signed.0.to_lowercase(), expected_hex.trim().to_lowercase());
}
```

- [ ] **Step 3: Implement signing**

`crates/jova-core-chains/src/sol/tx.rs`:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{SigningKey, Signer};
use jova_core_primitives::Ed25519Xprv;
use solana_message::VersionedMessage;
use solana_signature::Signature as SolSignature;
use solana_transaction::versioned::VersionedTransaction;

use crate::error::ChainError;

pub fn sign_sol_tx(
    xprv: &Ed25519Xprv,
    message_base64: &str,
    recent_blockhash: &str,
) -> Result<(String, String), ChainError> {
    // Decode the message.
    let bytes = B64.decode(message_base64.trim())
        .map_err(|_| ChainError::MalformedUnsignedTx("sol_invalid_base64".into()))?;
    let mut message: VersionedMessage = bincode::deserialize(&bytes)
        .map_err(|_| ChainError::MalformedUnsignedTx("sol_message_unsupported_version".into()))?;

    // Validate blockhash consistency.
    let claimed_blockhash = message.recent_blockhash();
    let expected = recent_blockhash.parse::<solana_pubkey::Pubkey>()
        .map_err(|_| ChainError::MalformedUnsignedTx("sol_invalid_recent_blockhash".into()))?;
    // VersionedMessage::recent_blockhash returns a Hash, not Pubkey; adjust:
    if claimed_blockhash.to_string() != recent_blockhash.trim() {
        return Err(ChainError::MalformedUnsignedTx("sol_blockhash_mismatch".into()));
    }

    // Sign the serialized message bytes.
    let signing_key = SigningKey::from_bytes(xprv.secret_bytes());
    let to_sign = message.serialize();
    let sig: ed25519_dalek::Signature = signing_key.sign(&to_sign);
    let sol_sig = SolSignature::from(sig.to_bytes());

    // Build the VersionedTransaction.
    let tx = VersionedTransaction {
        signatures: vec![sol_sig],
        message,
    };
    let wire = bincode::serialize(&tx)
        .map_err(|e| ChainError::SigningFailed(format!("sol_serialize_failed:{:?}", e)))?;
    let signature_b58 = bs58::encode(sol_sig.as_ref()).into_string();
    Ok((hex::encode(&wire), signature_b58))
}
```

(Some of the `solana-message` and `solana-transaction` crate types and method names may vary across versions; the test in Step 2 with the `solana-cli` reference output is the contract.)

- [ ] **Step 4: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test sol_tx
```

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-chains/ tools/sol-vector-capture/
git commit -m "feat(chains/sol): VersionedTransaction (v0) signing"
```

### Task 3c.4: Solana raw message signing

**Files:**
- Create: `crates/jova-core-chains/src/sol/message.rs`
- Modify: `crates/jova-core-chains/src/sol/mod.rs`
- Create: `crates/jova-core-chains/tests/sol_message.rs`

- [ ] **Step 1: Implement and test**

```rust
// crates/jova-core-chains/src/sol/message.rs
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use ed25519_dalek::{SigningKey, Signer};
use jova_core_primitives::Ed25519Xprv;

use crate::error::ChainError;

pub fn sign_sol_message(xprv: &Ed25519Xprv, message_base64: &str) -> Result<String, ChainError> {
    let message = B64.decode(message_base64.trim())
        .map_err(|_| ChainError::MalformedSignableMessage("sol_invalid_base64".into()))?;
    let signing_key = SigningKey::from_bytes(xprv.secret_bytes());
    let sig: ed25519_dalek::Signature = signing_key.sign(&message);
    Ok(bs58::encode(sig.to_bytes()).into_string())
}
```

Capture a reference signature with `solana-keygen sign` against the same seed; assert byte-identical.

- [ ] **Step 2: Commit**

```bash
git add crates/jova-core-chains/ tools/sol-vector-capture/
git commit -m "feat(chains/sol): raw ed25519 message signing"
```

### Task 3c.5: SolSigner trait + JovaWallet dispatch

Same pattern as XRP. Append `SolSigner` to `sol/mod.rs`, wire into `JovaWallet::sign_tx`, `sign_message`, `address`. Use derivation path `m/44'/501'/0'/0'`. Use `derive_ed25519` (not `derive_secp256k1`).

```bash
cargo build --workspace && cargo test --workspace
git add . && git commit -m "feat(core): SOL dispatch in JovaWallet"
```

### Task 3c.6: SOL vectors + parity tests

9 vectors total: 3 address (different account indices), 3 sign_tx (legacy-shaped v0, ALT-using, large message), 1 sign_message, 2 errors (`sol_invalid_base64`, `sol_blockhash_mismatch`).

Parity tests on Rust + Swift + Kotlin (mirror BTC pattern).

```bash
git add spec/test-vectors.json crates/jova-core/tests/vectors_sol.rs bindings/
git commit -m "test: SOL vector parity Rust + Swift + Kotlin"
```

---

## Task 3d (final): Tag v0.5.0

After all three sub-phases land:

- [ ] **Step 1: Confirm all CI workflows green on `main`**

```bash
gh run list --branch main --limit 6
```

All recent runs should be green.

- [ ] **Step 2: Tag**

```bash
git tag -a v0.5.0 -m "v0.5.0 — Phase 3 complete (every v1 chain shipping)"
git push origin v0.5.0
```

This is the SDK version that Phase 4 (app integration) consumes.

---

## Self-review

- [ ] Every sub-phase has TDD-level tasks.
- [ ] Vectors have real reference values from `cast` / `xrpl-py` / `solana-cli` / `bdk-cli`.
- [ ] Differential test for XRP (against `xrpl-py`) passes 100/100.
- [ ] SLIP-10 derivation tested against `solana-keygen pubkey` (independent reference).
- [ ] Anza split crates used (no active dependency on monolithic `solana-sdk`).
- [ ] WASM compile-smoke remains green throughout.
- [ ] All vectors pass on Rust + Swift + Kotlin.

---

## What this plan does NOT do

- Does not run WASM functional tests for SOL/XRP. Phase 6.
- Does not add Solana token (SPL) signing — only system program transfers and arbitrary v0 messages. Apps construct the message; SDK signs.
- Does not add XRP escrow / payment channel / NFTokenMint. Only Payment + OfferCreate are vector-tested. Other transaction types should work via the canonical signer but aren't proven by vectors until apps need them.

---

## Estimated time

3–5 weeks for sub-agents working in parallel. Sequential: 5–6 weeks. Time sinks:
1. solana-cli sign-only tooling reliability — local validator setup.
2. SLIP-10 derivation cross-checks.
3. xrpl-py bridging for differential test (BIP-39 seed → xrpl Wallet).
4. Anza split crates' API stability (the spike report's findings here are load-bearing).
