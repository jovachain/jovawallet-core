# Phase 2: Bitcoin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bitcoin native SegWit (BIP-84) addresses, BIP-174 PSBT signing for single-input / multi-input / multi-party scenarios, BIP-322 message signing with legacy `signMessage` fallback. Byte-identical vector parity across Rust + Swift + Kotlin. Tag `v0.2.0`.

**Architecture:** New `crates/jova-core-chains/src/btc/` module implementing the existing `ChainSigner` trait. BIP-84 derivation in `jova-core-primitives`. PSBT operations via `bdk_wallet`. Address encoding via `bitcoin` (rust-bitcoin). Signed-tx-or-updated-PSBT result encoding lets multi-party flows return without finalizing. Vectors append to `spec/test-vectors.json`; the existing test harness loads them on every binding.

**Tech Stack:** As Phase 1, plus `bdk_wallet` and `bitcoin` (rust-bitcoin) workspace deps already declared in Phase 0's `[workspace.dependencies]`.

**Why BTC second, not last:** Highest funds-on-chain risk and migration risk. Existing Android users hold real BTC at BIP-84 addresses; a derivation or signing bug means lost funds. Doing the dangerous thing while attention is fresh and the team is unfatigued.

**Preconditions:**
- Phase 1 complete; `v0.1.0` tagged.
- `ChainSigner` trait stable in `jova-core-chains/src/signer.rs`.
- Typed FFI shape established (`UnsignedTx::Bitcoin { psbt_base64 }` and `SignableMessage::Bitcoin { ... }` are already defined in the FFI enum even though Phase 1 returns `UnsupportedChain` for them).
- `JovaError` reason vocabulary frozen at v0.1.0 in `spec/errors.md`.
- `bdk_wallet 1.5+` and `bitcoin 0.33+` confirmed compiling natively in Phase -1's feasibility report.
- The Android app team has provided `tools/btc-migration-check/known-android-mappings.csv` with at least 100 known seed → `bc1q…` mappings exported from production storage. (Coordinate with the app team; do not start Phase 2 until this file exists.)

**Exit criteria:**
- 9+ BTC vectors green on Rust + Swift + Kotlin: 3 address (account 0/1/5 across 2 mnemonics), 3 sign_tx (single-input PSBT, multi-input PSBT, multi-party PSBT returning unfinalized), 1 BIP-322 sign_message, 1 legacy sign_message, 3 error vectors.
- BIP-84 official test vectors pass byte-identically on every binding (vectors imported from `bitcoin/bips/bip-0084`).
- `cargo run -p tools/btc-migration-check` reports 100/100 spot-check matches against the legacy Android storage.
- A SDK-signed PSBT broadcasts and confirms on Bitcoin mainnet at small amount (manual smoke test by the engineer driving Phase 2; document tx hash in `docs/btc-mainnet-smoke.md`).
- WASM compile-smoke remains green (no functional WASM coverage yet).
- Tag `v0.2.0` on `main`.

---

## Task 1: BIP-84 derivation in jova-core-primitives

**Files:**
- Modify: `crates/jova-core-primitives/src/path.rs`
- Modify: `crates/jova-core-primitives/src/lib.rs`
- Create: `crates/jova-core-primitives/tests/bip84.rs`

- [ ] **Step 1: Write the failing test against BIP-84 official vectors**

`crates/jova-core-primitives/tests/bip84.rs`:

```rust
use jova_core_primitives::{Mnemonic, DerivationPath, derive_secp256k1};

// BIP-84 official test vector mnemonic: 12 words
// Source: https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki
const BIP84_VECTOR_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn bip84_account0_external_index0_pubkey() {
    let seed = Mnemonic::to_seed(BIP84_VECTOR_MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    let xprv = derive_secp256k1(&seed, &path).unwrap();
    let pubkey_compressed = xprv.public_key_compressed();
    // Expected from BIP-84 vectors: 0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c
    let expected_hex = "0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c";
    assert_eq!(hex::encode(pubkey_compressed), expected_hex);
}

#[test]
fn bip84_path_helper() {
    let p = DerivationPath::bip84_path(0, 0, 0).unwrap();
    let parsed = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    assert_eq!(p, parsed);
}
```

- [ ] **Step 2: Run; confirm it fails**

```bash
cargo test -p jova-core-primitives --test bip84
```

Expected: `error[E0599]: no function or associated item named 'bip84_path' found for struct 'DerivationPath'`.

- [ ] **Step 3: Add the BIP-84 path helper**

Modify `crates/jova-core-primitives/src/path.rs` — add at the bottom of `impl DerivationPath`:

```rust
impl DerivationPath {
    /// Build a BIP-84 path: m/84'/coin_type'/account'/change/index. Coin type
    /// is hardcoded to 0 (Bitcoin mainnet) for the helper.
    pub fn bip84_path(account: u32, change: u32, index: u32) -> Result<Self, PathError> {
        if account >= HARDENED_OFFSET || change >= HARDENED_OFFSET || index >= HARDENED_OFFSET {
            return Err(PathError::IndexOutOfRange);
        }
        Ok(Self {
            indices: alloc::vec![
                84 + HARDENED_OFFSET,
                0 + HARDENED_OFFSET,         // BTC mainnet coin type
                account + HARDENED_OFFSET,
                change,
                index,
            ],
        })
    }
}
```

- [ ] **Step 4: Run; confirm it passes**

```bash
cargo test -p jova-core-primitives --test bip84
```

Expected: both tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-primitives/
git commit -m "feat(primitives): BIP-84 derivation path helper + BIP-84 official vector"
```

---

## Task 2: P2WPKH address derivation (bech32)

**Files:**
- Create: `crates/jova-core-chains/src/btc/mod.rs`
- Create: `crates/jova-core-chains/src/btc/address.rs`
- Modify: `crates/jova-core-chains/src/lib.rs`
- Create: `crates/jova-core-chains/tests/btc_address.rs`

- [ ] **Step 1: Write the failing test**

`crates/jova-core-chains/tests/btc_address.rs`:

```rust
use jova_core_chains::btc::{derive_p2wpkh, validate_btc_address};
use jova_core_primitives::{Mnemonic, DerivationPath, derive_secp256k1};

const BIP84_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn bip84_first_address_matches_official_vector() {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    let xprv = derive_secp256k1(&seed, &path).unwrap();
    let addr = derive_p2wpkh(&xprv);
    // BIP-84 official vector first address:
    assert_eq!(addr, "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
}

#[test]
fn bip84_second_address_matches_official_vector() {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/1").unwrap();
    let xprv = derive_secp256k1(&seed, &path).unwrap();
    let addr = derive_p2wpkh(&xprv);
    assert_eq!(addr, "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g");
}

#[test]
fn validates_known_bech32_addresses() {
    assert!(validate_btc_address("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"));
    assert!(validate_btc_address("bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g"));
}

#[test]
fn rejects_malformed_addresses() {
    assert!(!validate_btc_address("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyx"));   // bad checksum
    assert!(!validate_btc_address("bc1q"));                                           // too short
    assert!(!validate_btc_address("1NhMV5x8d4owZ3p3HHo2tQrEoWpJK7tnvm"));             // P2PKH (legacy)
    assert!(!validate_btc_address("not-an-address"));
}
```

- [ ] **Step 2: Run; confirm fails to compile**

```bash
cargo test -p jova-core-chains --test btc_address
```

Expected: `error[E0432]: unresolved import jova_core_chains::btc`.

- [ ] **Step 3: Implement the address module**

`crates/jova-core-chains/src/btc/address.rs`:

```rust
use bitcoin::{Address, Network, PublicKey, CompressedPublicKey};
use jova_core_primitives::XPrv;

/// Derive a P2WPKH (BIP-84 native SegWit) address from a derived XPrv.
/// Returns the canonical bech32 string with `bc1q` prefix.
pub fn derive_p2wpkh(xprv: &XPrv) -> String {
    let compressed_bytes = xprv.public_key_compressed();
    let pk = PublicKey::from_slice(&compressed_bytes)
        .expect("32-byte secp256k1 public key from xprv is always valid");
    let cpk = CompressedPublicKey::try_from(pk)
        .expect("compressed pubkey roundtrip");
    let address = Address::p2wpkh(&cpk, Network::Bitcoin);
    address.to_string()
}

/// Validate that a string is a canonical Bitcoin mainnet address.
/// In v1 we accept only P2WPKH (BIP-84) addresses on mainnet — legacy P2PKH
/// (`1…`) and Taproot (`bc1p…`) are rejected. Phase 2 supports BIP-84 only.
pub fn validate_btc_address(s: &str) -> bool {
    let parsed = match Address::from_str(s).and_then(|a| a.require_network(Network::Bitcoin).map_err(|e| e.into())) {
        Ok(a) => a,
        Err(_) => return false,
    };
    matches!(parsed.address_type(), Some(bitcoin::AddressType::P2wpkh))
}

use std::str::FromStr;
```

`crates/jova-core-chains/src/btc/mod.rs`:

```rust
pub mod address;

pub use address::{derive_p2wpkh, validate_btc_address};
```

- [ ] **Step 4: Wire into `lib.rs`**

Modify `crates/jova-core-chains/src/lib.rs`. Add:

```rust
pub mod btc;
```

(Keep next to the existing `pub mod evm;` declaration.)

- [ ] **Step 5: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test btc_address
```

Expected: all four tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/jova-core-chains/
git commit -m "feat(chains/btc): P2WPKH address derivation + bech32 validation"
```

---

## Task 3: PSBT signing — single-input case first

**Files:**
- Create: `crates/jova-core-chains/src/btc/psbt.rs`
- Modify: `crates/jova-core-chains/src/btc/mod.rs`
- Create: `crates/jova-core-chains/tests/btc_psbt_single.rs`

- [ ] **Step 1: Write the failing test**

The simplest BTC signing test: a PSBT with one input that the wallet's BIP-84 key can sign. We construct the unsigned PSBT in test setup using `bdk_wallet`, then assert the signed tx hex matches a known-good output (captured from `bdk-cli` against the same seed).

`crates/jova-core-chains/tests/btc_psbt_single.rs`:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use jova_core_chains::btc::sign_psbt;
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const BIP84_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// PSBT base64 captured from bdk-cli for a single-input transfer:
//   - Input: 100,000 sats from m/84'/0'/0'/0/0
//   - Output: 90,000 sats to bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g
//   - Fee: 10,000 sats
// (Generated locally; see tools/btc-vector-capture/single_input.sh)
const UNSIGNED_PSBT_B64: &str = "cHNidP8BAHECAAAAA..."; // truncated for brevity in this plan; full value populated by capture

const EXPECTED_SIGNED_HEX: &str = "0100000000010109e6...";  // captured from same flow finalized

#[test]
fn signs_single_input_psbt_to_finalized_tx() {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    let xprv = derive_secp256k1(&seed, &path).unwrap();

    let result = sign_psbt(&xprv, UNSIGNED_PSBT_B64).expect("signs");

    // Result is either a finalized tx (hex) or an updated PSBT (base64).
    // For single-input where we own the key, expect finalized.
    assert!(result.is_finalized);
    assert_eq!(result.tx_hex.unwrap().to_lowercase(), EXPECTED_SIGNED_HEX.to_lowercase());
}
```

(The `UNSIGNED_PSBT_B64` and `EXPECTED_SIGNED_HEX` strings are placeholders only as far as the plan body goes — Step 2 below describes the capture procedure that produces real values for the test file. The agent runs the capture before the test compiles, so the values are real by the time tests run.)

- [ ] **Step 2: Capture reference values**

Install `bdk-cli`:

```bash
cargo install bdk-cli --version 1.0 --locked
```

Create `tools/btc-vector-capture/single_input.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
DESCRIPTOR="wpkh([5c9e228d/84'/0'/0']tpubD..../0/*)"   # bdk-cli will print this from the mnemonic

# Use bdk-cli to construct an unsigned PSBT against a regtest UTXO set primed
# with a known 100k-sat output to address index 0.
bdk-cli wallet --network regtest sync
PSBT=$(bdk-cli wallet --network regtest create_tx --to "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g:90000" --send_all --fee_rate 1.0 | jq -r '.psbt')

echo "UNSIGNED_PSBT_B64=$PSBT"

# Sign and finalize via bdk-cli for the reference signed_hex.
SIGNED_PSBT=$(bdk-cli wallet --network regtest sign --psbt "$PSBT" | jq -r '.psbt')
TX=$(bdk-cli wallet --network regtest finalize_psbt --psbt "$SIGNED_PSBT" | jq -r '.tx')

echo "EXPECTED_SIGNED_HEX=$TX"
```

```bash
chmod +x tools/btc-vector-capture/single_input.sh
./tools/btc-vector-capture/single_input.sh
```

Paste the captured `UNSIGNED_PSBT_B64` and `EXPECTED_SIGNED_HEX` values into the test file from Step 1, replacing the truncated placeholders. **The test file does not commit until both values are real captures.**

- [ ] **Step 3: Run; confirm fails**

```bash
cargo test -p jova-core-chains --test btc_psbt_single
```

Expected: `unresolved import jova_core_chains::btc::sign_psbt`.

- [ ] **Step 4: Implement single-input PSBT signing**

`crates/jova-core-chains/src/btc/psbt.rs`:

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use bdk_wallet::{psbt::PsbtUtils, KeychainKind};
use bitcoin::{
    Psbt,
    psbt::{Input as PsbtInput, PartiallySignedTransaction},
    secp256k1::{Secp256k1, SecretKey, Message},
    EcdsaSighashType,
};
use jova_core_primitives::XPrv;

use crate::error::ChainError;

/// Result of signing a PSBT. If every input was signable by this wallet's key
/// (single-party flow), `is_finalized` is true and `tx_hex` carries the
/// broadcast-ready transaction. If at least one input requires another
/// signer's contribution, `is_finalized` is false and `psbt_base64` carries
/// the updated PSBT for the next signer.
pub struct PsbtSignResult {
    pub is_finalized: bool,
    pub tx_hex: Option<String>,
    pub psbt_base64: Option<String>,
    /// sha256d of the finalized tx, present only when finalized.
    pub tx_hash: Option<String>,
}

pub fn sign_psbt(xprv: &XPrv, psbt_base64: &str) -> Result<PsbtSignResult, ChainError> {
    // 1. Decode the PSBT.
    let bytes = B64.decode(psbt_base64)
        .map_err(|_| ChainError::MalformedUnsignedTx("psbt_invalid_base64".into()))?;
    let mut psbt = Psbt::deserialize(&bytes)
        .map_err(|_| ChainError::MalformedUnsignedTx("psbt_invalid_serialization".into()))?;

    // 2. For each input, attempt to sign with the supplied xprv if the input's
    //    BIP-32 derivation hint matches the wallet's path. (Phase 2 supports
    //    only the simplest case: input has bip32_derivation field.)
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("secp256k1_invalid_secret".into()))?;

    let mut signed_count = 0usize;
    for (i, input) in psbt.inputs.iter_mut().enumerate() {
        if try_sign_input(&secp, &sk, &mut psbt.unsigned_tx, i, input)? {
            signed_count += 1;
        }
    }

    if signed_count == 0 {
        return Err(ChainError::MalformedUnsignedTx("psbt_no_signable_inputs".into()));
    }

    // 3. Try to finalize. If every input is signed, we get back the raw tx.
    //    Otherwise, return the updated PSBT for downstream signers.
    match psbt.finalize_mut(&secp) {
        Ok(()) => {
            let tx = psbt.extract_tx_unchecked_fee_rate();
            let tx_bytes = bitcoin::consensus::encode::serialize(&tx);
            let tx_hash = tx.compute_txid().to_string();
            Ok(PsbtSignResult {
                is_finalized: true,
                tx_hex: Some(hex::encode(tx_bytes)),
                psbt_base64: None,
                tx_hash: Some(tx_hash),
            })
        }
        Err(_) => {
            // Multi-party flow: return updated PSBT.
            let updated = psbt.serialize();
            Ok(PsbtSignResult {
                is_finalized: false,
                tx_hex: None,
                psbt_base64: Some(B64.encode(updated)),
                tx_hash: None,
            })
        }
    }
}

/// Sign one PSBT input if our key matches the input's witness program.
/// Returns `Ok(true)` if signed, `Ok(false)` if the input's key isn't ours.
fn try_sign_input(
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    sk: &SecretKey,
    unsigned_tx: &mut bitcoin::Transaction,
    input_idx: usize,
    input: &mut PsbtInput,
) -> Result<bool, ChainError> {
    use bitcoin::sighash::{EcdsaSighashType, SighashCache};

    // Compute our compressed pubkey to compare against the input's BIP-32
    // derivation hint or witness UTXO scriptPubKey.
    let our_pk = bitcoin::PublicKey::from_private_key(secp, &bitcoin::PrivateKey::new(*sk, bitcoin::Network::Bitcoin));

    // For BIP-84 P2WPKH, the witness UTXO's scriptPubKey contains hash160(pubkey).
    let witness_utxo = match &input.witness_utxo {
        Some(w) => w,
        None => return Ok(false),   // Phase 2 only supports witness_utxo-bearing inputs.
    };

    let expected_program = bitcoin::WPubkeyHash::from(&our_pk.try_into().map_err(|_| ChainError::SigningFailed("pubkey_invalid".into()))?);
    if !witness_utxo.script_pubkey.is_p2wpkh()
        || witness_utxo.script_pubkey.as_bytes()[2..] != *expected_program.as_byte_array() {
        return Ok(false);   // Not our key.
    }

    // Compute sighash and sign.
    let mut sighasher = SighashCache::new(unsigned_tx);
    let sighash = sighasher.p2wpkh_signature_hash(
        input_idx,
        &witness_utxo.script_pubkey,
        witness_utxo.value,
        EcdsaSighashType::All,
    ).map_err(|_| ChainError::SigningFailed("sighash_compute_failed".into()))?;

    let msg = Message::from_digest(sighash.to_byte_array());
    let sig = secp.sign_ecdsa(&msg, sk);
    let sig_with_type = bitcoin::ecdsa::Signature {
        signature: sig,
        sighash_type: EcdsaSighashType::All,
    };

    input.partial_sigs.insert(our_pk, sig_with_type);
    Ok(true)
}
```

(The exact `bdk_wallet 1.5` / `bitcoin 0.33` API may differ in minor ways from this draft — function names like `compute_txid`, `extract_tx_unchecked_fee_rate`, `is_p2wpkh`, etc. were stabilized in those versions. If the snippet doesn't compile, the **test from Step 1 is the contract**: adjust the function calls to whatever the crate uses, the captured `EXPECTED_SIGNED_HEX` is invariant.)

Update `crates/jova-core-chains/src/btc/mod.rs`:

```rust
pub mod address;
pub mod psbt;

pub use address::{derive_p2wpkh, validate_btc_address};
pub use psbt::{sign_psbt, PsbtSignResult};
```

- [ ] **Step 5: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test btc_psbt_single
```

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add crates/jova-core-chains/ tools/btc-vector-capture/
git commit -m "feat(chains/btc): single-input PSBT signing"
```

---

## Task 4: PSBT signing — multi-input + multi-party cases

**Files:**
- Modify: `crates/jova-core-chains/src/btc/psbt.rs`
- Create: `crates/jova-core-chains/tests/btc_psbt_multi.rs`
- Create: `tools/btc-vector-capture/multi_input.sh`
- Create: `tools/btc-vector-capture/multi_party.sh`

- [ ] **Step 1: Capture multi-input PSBT vectors**

Adapt `tools/btc-vector-capture/single_input.sh` into `multi_input.sh`. Differences: prime two UTXOs at indices 0 and 1, construct a PSBT consuming both, sign with the same wallet (which owns both keys). Output a finalized tx.

```bash
chmod +x tools/btc-vector-capture/multi_input.sh
./tools/btc-vector-capture/multi_input.sh
# Capture UNSIGNED_PSBT_B64_MULTI and EXPECTED_SIGNED_HEX_MULTI
```

- [ ] **Step 2: Capture multi-party PSBT vector**

`tools/btc-vector-capture/multi_party.sh`: prime two UTXOs, one belonging to wallet A (our SDK), one belonging to wallet B (a separate test wallet). Construct a 2-input PSBT. Sign with A only via SDK. Expected: result is **NOT finalized**; the returned PSBT contains A's partial signature on input 0, with input 1 still needing B's signature.

```bash
./tools/btc-vector-capture/multi_party.sh
# Capture UNSIGNED_PSBT_B64_TWOPARTY and EXPECTED_PSBT_AFTER_A_SIGNS
```

- [ ] **Step 3: Write the failing tests**

`crates/jova-core-chains/tests/btc_psbt_multi.rs`:

```rust
use jova_core_chains::btc::sign_psbt;
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const BIP84_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const UNSIGNED_PSBT_B64_MULTI: &str = include_str!("../../../tools/btc-vector-capture/captures/multi_input.psbt.b64");
const EXPECTED_SIGNED_HEX_MULTI: &str = include_str!("../../../tools/btc-vector-capture/captures/multi_input.signed_hex");
const UNSIGNED_PSBT_B64_TWOPARTY: &str = include_str!("../../../tools/btc-vector-capture/captures/two_party.psbt.b64");
const EXPECTED_PSBT_AFTER_A_SIGNS: &str = include_str!("../../../tools/btc-vector-capture/captures/two_party.after_a.psbt.b64");

fn xprv() -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    derive_secp256k1(&seed, &path).unwrap()
}

#[test]
fn signs_multi_input_psbt_we_own_all_inputs_to_finalized() {
    let result = sign_psbt(&xprv(), UNSIGNED_PSBT_B64_MULTI.trim()).unwrap();
    assert!(result.is_finalized);
    assert_eq!(result.tx_hex.unwrap().trim().to_lowercase(),
               EXPECTED_SIGNED_HEX_MULTI.trim().to_lowercase());
}

#[test]
fn multi_party_psbt_returns_unfinalized_after_our_signature() {
    let result = sign_psbt(&xprv(), UNSIGNED_PSBT_B64_TWOPARTY.trim()).unwrap();
    assert!(!result.is_finalized);
    assert_eq!(result.psbt_base64.unwrap().trim(),
               EXPECTED_PSBT_AFTER_A_SIGNS.trim());
}
```

- [ ] **Step 4: Run; confirm passes**

The PSBT signing function from Task 3 already handles both cases (the finalize step decides). Run:

```bash
cargo test -p jova-core-chains --test btc_psbt_multi
```

Expected: both tests pass without code changes.

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-chains/tests/btc_psbt_multi.rs tools/btc-vector-capture/
git commit -m "test(chains/btc): multi-input + multi-party PSBT signing"
```

---

## Task 5: BIP-322 message signing + legacy `signMessage` fallback

**Files:**
- Create: `crates/jova-core-chains/src/btc/message.rs`
- Modify: `crates/jova-core-chains/src/btc/mod.rs`
- Create: `crates/jova-core-chains/tests/btc_message.rs`
- Create: `tools/btc-vector-capture/messages.sh`

- [ ] **Step 1: Capture message-signing vectors**

`tools/btc-vector-capture/messages.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
ADDRESS="bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
MESSAGE="Hello, Jova"

# BIP-322 simple signature via bdk-cli (or via the bip322 reference crate
# if bdk-cli doesn't expose it directly).
BIP322_SIG=$(bdk-cli sign_message --message "$MESSAGE" --address "$ADDRESS" --mnemonic "$MNEMONIC" --scheme bip322)
echo "BIP322_SIG=$BIP322_SIG"

# Legacy signMessage (Bitcoin Core RPC compatible) — uses the
# "\x18Bitcoin Signed Message:\n" prefix scheme.
LEGACY_SIG=$(bdk-cli sign_message --message "$MESSAGE" --address "$ADDRESS" --mnemonic "$MNEMONIC" --scheme legacy)
echo "LEGACY_SIG=$LEGACY_SIG"
```

```bash
./tools/btc-vector-capture/messages.sh
# Capture both sigs into captures/bip322_sig.txt and captures/legacy_sig.txt
```

- [ ] **Step 2: Write the failing tests**

`crates/jova-core-chains/tests/btc_message.rs`:

```rust
use jova_core_chains::btc::{sign_btc_message, BtcMsgScheme};
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const MESSAGE: &str = "Hello, Jova";
const EXPECTED_BIP322_SIG: &str = include_str!("../../../tools/btc-vector-capture/captures/bip322_sig.txt");
const EXPECTED_LEGACY_SIG: &str = include_str!("../../../tools/btc-vector-capture/captures/legacy_sig.txt");

fn xprv() -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(MNEMONIC, "").unwrap();
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").unwrap();
    derive_secp256k1(&seed, &path).unwrap()
}

#[test]
fn bip322_signature_matches_reference() {
    let sig = sign_btc_message(&xprv(), MESSAGE, ADDRESS, BtcMsgScheme::Bip322).unwrap();
    assert_eq!(sig.trim(), EXPECTED_BIP322_SIG.trim());
}

#[test]
fn legacy_signature_matches_reference() {
    let sig = sign_btc_message(&xprv(), MESSAGE, ADDRESS, BtcMsgScheme::Legacy).unwrap();
    assert_eq!(sig.trim(), EXPECTED_LEGACY_SIG.trim());
}

#[test]
fn rejects_address_not_owned_by_wallet() {
    // A real Bitcoin address that this seed doesn't derive.
    let foreign = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let result = sign_btc_message(&xprv(), MESSAGE, foreign, BtcMsgScheme::Bip322);
    assert!(matches!(result, Err(ref e) if e.to_string().contains("btc_message_address_mismatch")));
}
```

- [ ] **Step 3: Implement message signing**

`crates/jova-core-chains/src/btc/message.rs`:

```rust
use bitcoin::{
    Address, Network, PublicKey, CompressedPublicKey,
    secp256k1::{Secp256k1, SecretKey, Message as Secp256k1Message},
    sign_message::{MessageSignature, signed_msg_hash},
};
use jova_core_primitives::XPrv;
use std::str::FromStr;

use crate::error::ChainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtcMsgScheme {
    Bip322,
    Legacy,
}

pub fn sign_btc_message(
    xprv: &XPrv,
    message: &str,
    address: &str,
    scheme: BtcMsgScheme,
) -> Result<String, ChainError> {
    // Verify the address corresponds to this xprv's pubkey.
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("secp256k1_invalid_secret".into()))?;
    let pk = bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(sk, Network::Bitcoin));
    let cpk = CompressedPublicKey::try_from(pk)
        .map_err(|_| ChainError::SigningFailed("pubkey_compress_failed".into()))?;
    let derived_address = Address::p2wpkh(&cpk, Network::Bitcoin);
    if derived_address.to_string() != address {
        return Err(ChainError::MalformedSignableMessage("btc_message_address_mismatch".into()));
    }

    match scheme {
        BtcMsgScheme::Bip322 => sign_bip322(&secp, &sk, message, &derived_address),
        BtcMsgScheme::Legacy => sign_legacy(&secp, &sk, message),
    }
}

/// BIP-322 simple signature: construct the to_spend / to_sign virtual transactions
/// and sign per the BIP. Returns base64-encoded witness.
fn sign_bip322(
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    sk: &SecretKey,
    message: &str,
    address: &Address,
) -> Result<String, ChainError> {
    use bitcoin::bip322;
    let signed = bip322::sign_simple(address, message, sk, secp)
        .map_err(|_| ChainError::SigningFailed("bip322_sign_failed".into()))?;
    Ok(signed.encode_to_base64())
}

/// Legacy signMessage as used by Bitcoin Core's RPC. Prepends
/// "\x18Bitcoin Signed Message:\n<len>" and signs the double-SHA256 digest.
fn sign_legacy(
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    sk: &SecretKey,
    message: &str,
) -> Result<String, ChainError> {
    let hash = signed_msg_hash(message);
    let msg = Secp256k1Message::from_digest(hash.to_byte_array());
    let sig = secp.sign_ecdsa_recoverable(&msg, sk);
    let msg_sig = MessageSignature {
        signature: sig.to_standard(),
        compressed: true,
    };
    Ok(msg_sig.to_base64())
}
```

(Function paths in `bitcoin` 0.33 may differ — `bip322::sign_simple`, `MessageSignature::to_base64`, `signed_msg_hash`. The tests' captured reference values are the contract.)

Update `crates/jova-core-chains/src/btc/mod.rs`:

```rust
pub mod address;
pub mod message;
pub mod psbt;

pub use address::{derive_p2wpkh, validate_btc_address};
pub use message::{sign_btc_message, BtcMsgScheme};
pub use psbt::{sign_psbt, PsbtSignResult};
```

- [ ] **Step 4: Run; confirm passes**

```bash
cargo test -p jova-core-chains --test btc_message
```

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core-chains/ tools/btc-vector-capture/
git commit -m "feat(chains/btc): BIP-322 + legacy signMessage with reference captures"
```

---

## Task 6: BtcSigner trait impl + JovaWallet dispatch

**Files:**
- Modify: `crates/jova-core-chains/src/btc/mod.rs`
- Modify: `crates/jova-core/src/wallet.rs`
- Modify: `crates/jova-core/src/chain.rs`

- [ ] **Step 1: Implement `ChainSigner` for `BtcSigner`**

Append to `crates/jova-core-chains/src/btc/mod.rs`:

```rust
use crate::address::{Address as JovaAddress, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;
use jova_core_primitives::XPrv;

pub struct BtcSigner;

impl ChainSigner for BtcSigner {
    fn derive_address(&self, key: &XPrv) -> Result<JovaAddress, ChainError> {
        Ok(JovaAddress {
            chain: "bitcoin".to_string(),
            value: derive_p2wpkh(key),
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        validate_btc_address(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Bitcoin { psbt_base64 } => {
                let result = sign_psbt(key, psbt_base64)?;
                let raw = if result.is_finalized {
                    result.tx_hex.unwrap()
                } else {
                    // Multi-party flow: return updated PSBT base64 in raw_hex.
                    // Apps inspect is_finalized via prefix? No — we use a
                    // distinguished prefix to communicate state.
                    format!("psbt:{}", result.psbt_base64.unwrap())
                };
                Ok(SignedTx {
                    chain: "bitcoin".to_string(),
                    raw_hex: raw,
                    tx_hash: result.tx_hash.unwrap_or_default(),
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx("expected_bitcoin_variant".into())),
        }
    }

    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError> {
        match msg {
            SignableMessage::Bitcoin { message, address, scheme } => {
                let scheme_internal = match scheme {
                    crate::signable_message::BtcMsgScheme::Bip322 => BtcMsgScheme::Bip322,
                    crate::signable_message::BtcMsgScheme::Legacy => BtcMsgScheme::Legacy,
                };
                let hex = sign_btc_message(key, message, address, scheme_internal)?;
                Ok(Signature { hex })
            }
            _ => Err(ChainError::MalformedSignableMessage("expected_bitcoin_message".into())),
        }
    }
}
```

(The `format!("psbt:{}", ...)` prefix is a deliberate signal in `raw_hex` that this is an updated PSBT, not a finalized tx. Apps and `integration-ios.md` / `integration-android.md` document this convention — `raw_hex` starting with `psbt:` means "hand to next signer," anything else is broadcast-ready.)

- [ ] **Step 2: Add `Bitcoin` variant to `JovaChain` and wire into wallet**

Modify `crates/jova-core/src/chain.rs` — `Bitcoin` is already in the enum from Phase 1 (we added it for the typed FFI even though it wasn't usable). Verify:

```bash
grep -A2 "pub enum JovaChain" crates/jova-core/src/chain.rs
```

If `Bitcoin,` is missing, add it. Also add `derivation_path` and a `chain_label` for `Bitcoin`:

```rust
impl JovaChain {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            // ... EVM ones ...
            Self::Bitcoin => "bitcoin",
            // Phase 3 fills SOL and XRP.
        }
    }

    pub(crate) fn derivation_path(&self) -> &'static str {
        match self {
            // EVM chains share m/44'/60'/0'/0/0.
            Self::Bitcoin => "m/84'/0'/0'/0/0",
            // ... EVM ones return existing path ...
        }
    }
}
```

Modify `crates/jova-core/src/wallet.rs`:

```rust
use jova_core_chains::btc::BtcSigner;

// In JovaWallet::sign_tx, add the Bitcoin arm:
pub fn sign_tx(&self, unsigned: &UnsignedTx) -> Result<SignedTx, JovaError> {
    match unsigned {
        UnsignedTx::Evm(_) => { /* existing Phase 1 path */ }
        UnsignedTx::Bitcoin { .. } => {
            let xprv = self.derive_path("m/84'/0'/0'/0/0")?;
            Ok(BtcSigner.sign_tx(&xprv, unsigned)?)
        }
        _ => Err(JovaError::UnsupportedChain("phase2_evm_btc_only".into())),
    }
}

// In sign_message:
pub fn sign_message(&self, msg: &SignableMessage) -> Result<Signature, JovaError> {
    match msg {
        SignableMessage::EvmPersonalSign { .. } | SignableMessage::EvmTypedDataV4 { .. } => { /* existing */ }
        SignableMessage::Bitcoin { .. } => {
            let xprv = self.derive_path("m/84'/0'/0'/0/0")?;
            Ok(BtcSigner.sign_message(&xprv, msg)?)
        }
        _ => Err(JovaError::UnsupportedChain("phase2_evm_btc_only".into())),
    }
}

// In address(), the dispatch now handles Bitcoin via BtcSigner:
pub fn address(&self, chain: &JovaChain, _account: u32) -> Result<Address, JovaError> {
    match chain {
        JovaChain::Bitcoin => {
            let xprv = self.derive_path("m/84'/0'/0'/0/0")?;
            Ok(BtcSigner.derive_address(&xprv)?)
        }
        chain if chain.evm_chain_id().is_some() => { /* existing EVM path */ }
        _ => Err(JovaError::UnsupportedChain(format!("{:?}", chain))),
    }
}
```

- [ ] **Step 3: Build**

```bash
cargo build --workspace
```

- [ ] **Step 4: Commit**

```bash
git add crates/jova-core/ crates/jova-core-chains/
git commit -m "feat(core): BTC dispatch in JovaWallet via BtcSigner"
```

---

## Task 7: Author the full Phase 2 vector set

**Files:**
- Modify: `spec/test-vectors.json`
- Modify: `spec/errors.md`
- Modify: `tools/verify-spec/src/main.rs` (extend BTC reason vocabulary check)

- [ ] **Step 1: Append BTC reason vocabulary to spec/errors.md**

Add the following entries to `spec/errors.md` under "Reason vocabulary":

```markdown
### Bitcoin (`malformed_unsigned_tx` reasons)

| Reason | Means |
|---|---|
| `psbt_invalid_base64` | PSBT base64 decode failed |
| `psbt_invalid_serialization` | base64 decoded but PSBT structure malformed |
| `psbt_no_signable_inputs` | none of the PSBT inputs are signable by this wallet's key |
| `expected_bitcoin_variant` | Internal: routing mismatch |

### Bitcoin (`malformed_signable_message` reasons)

| Reason | Means |
|---|---|
| `btc_message_address_mismatch` | the supplied address does not correspond to the wallet's derived key |
| `btc_unsupported_scheme` | unknown BtcMsgScheme value |
| `expected_bitcoin_message` | Internal: routing mismatch |

### Bitcoin (`signing_failed` reasons)

| Reason | Means |
|---|---|
| `bip322_sign_failed` | bip322::sign_simple returned an error |
| `secp256k1_invalid_secret` | XPrv private key bytes rejected by secp256k1 |
| `pubkey_compress_failed` | uncompressed pubkey could not be compressed |
| `pubkey_invalid` | pubkey did not parse |
| `sighash_compute_failed` | bitcoin sighash computation returned an error |
```

- [ ] **Step 2: Author the 9+ vectors via the capture scripts**

The captures in Tasks 3, 4, 5 produced reference values for individual flows. Now consolidate into `spec/test-vectors.json`. Each vector follows the schema from Phase 0.

Vectors to add (input shapes are predetermined; expected values come from `tools/btc-vector-capture/captures/*`):

1. `btc.address.bip84_mnemonic_a_account0_index0` — abandon-about, account 0, index 0 → `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`
2. `btc.address.bip84_mnemonic_a_account0_index1` — same mnemonic, index 1 → `bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g`
3. `btc.address.bip84_mnemonic_a_account5_index0` — same mnemonic, account 5, index 0
4. `btc.address.bip84_mnemonic_b_account0_index0` — second BIP-39 mnemonic ("ozone drill...")
5. `btc.sign_tx.psbt_single_input` — captured in Task 3
6. `btc.sign_tx.psbt_multi_input_owned` — captured in Task 4
7. `btc.sign_tx.psbt_multi_party_partial` — captured in Task 4 (expected raw_hex starts with `psbt:`)
8. `btc.sign_message.bip322_simple` — captured in Task 5
9. `btc.sign_message.legacy_bitcoin_core` — captured in Task 5
10. `btc.error.psbt_invalid_base64` — input: `{"psbt_base64": "not-base64!"}`, expected error: MalformedUnsignedTx with reason `psbt_invalid_base64`
11. `btc.error.psbt_no_signable_inputs` — input: a real PSBT signing for a different wallet's keys; expected error: `psbt_no_signable_inputs`
12. `btc.error.btc_message_address_mismatch` — input: SignableMessage::Bitcoin with a foreign address; expected error: `btc_message_address_mismatch`

The agent extends `tools/btc-vector-capture/capture.sh` to emit the populated JSON for each vector and concatenate with the existing `spec/test-vectors.json` (incrementing `version` to `"0.3"`).

- [ ] **Step 3: Verify schema compliance**

```bash
cargo run -p jova-verify-spec
```

Expected: `verify-spec: OK`. The placeholder check (`TODO`/`<capture`/`REPLACE`) added in Phase 1 still applies — every BTC vector's `expected` field must be a real reference value.

- [ ] **Step 4: Commit**

```bash
git add spec/ tools/btc-vector-capture/
git commit -m "feat(spec): 12 BTC vectors covering BIP-84, PSBT, BIP-322, error paths"
```

---

## Task 8: Vector parity tests on every binding

**Files:**
- Create: `crates/jova-core/tests/vectors_btc.rs`
- Create: `bindings/swift/Tests/JovaCoreTests/BtcVectorsTests.swift`
- Create: `bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/BtcVectorsTest.kt`

- [ ] **Step 1: Rust vector test**

`crates/jova-core/tests/vectors_btc.rs`:

```rust
use jova_core::*;
use serde_json::Value;

fn load_vectors() -> Vec<Value> {
    let raw = include_str!("../../../spec/test-vectors.json");
    let v: Value = serde_json::from_str(raw).unwrap();
    v["vectors"].as_array().unwrap().clone()
}

#[test]
fn btc_address_vectors() {
    for v in load_vectors() {
        if v["kind"] != "address" { continue; }
        if v["input"]["chain"]["kind"] != "bitcoin" { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let expected = v["expected"]["address"].as_str().unwrap();

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();
        let got = wallet.address(&JovaChain::Bitcoin, 0).unwrap();
        assert_eq!(got.value, expected, "vector {}", v["id"]);
    }
}

#[test]
fn btc_sign_tx_vectors() {
    for v in load_vectors() {
        if v["kind"] != "sign_tx" { continue; }
        if v["input"]["unsigned_tx"]["kind"] != "bitcoin" { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone()).unwrap();
        let expected_hex = v["expected"]["signed_hex"].as_str().unwrap();

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();
        let signed = wallet.sign_tx(&unsigned).unwrap();
        assert_eq!(signed.raw_hex, expected_hex, "vector {}", v["id"]);
    }
}

#[test]
fn btc_sign_message_vectors() {
    for v in load_vectors() {
        if v["kind"] != "sign_message" { continue; }
        if v["input"]["message"]["kind"] != "bitcoin" { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let msg: SignableMessage = serde_json::from_value(v["input"]["message"].clone()).unwrap();
        let expected_hex = v["expected"]["signature_hex"].as_str().unwrap();

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();
        let sig = wallet.sign_message(&msg).unwrap();
        assert_eq!(sig.hex, expected_hex, "vector {}", v["id"]);
    }
}

#[test]
fn btc_error_vectors() {
    use jova_core::JovaError;
    for v in load_vectors() {
        if v["kind"] != "error" { continue; }
        let id = v["id"].as_str().unwrap();
        if !id.starts_with("btc.") { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();

        let expected_variant = v["expected"]["error_variant"].as_str().unwrap();
        let expected_reason  = v["expected"]["reason"].as_str();

        let result: Result<(), JovaError> = if v["input"].get("unsigned_tx").is_some() {
            let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone()).unwrap();
            wallet.sign_tx(&unsigned).map(|_| ())
        } else if v["input"].get("message").is_some() {
            let msg: SignableMessage = serde_json::from_value(v["input"]["message"].clone()).unwrap();
            wallet.sign_message(&msg).map(|_| ())
        } else {
            panic!("error vector must have unsigned_tx or message: {}", id);
        };

        let err = result.expect_err("vector should fail");
        match (&err, expected_variant, expected_reason) {
            (JovaError::MalformedUnsignedTx { reason }, "MalformedUnsignedTx", Some(expected)) => {
                assert_eq!(reason, expected, "vector {}", id);
            }
            (JovaError::MalformedSignableMessage { reason }, "MalformedSignableMessage", Some(expected)) => {
                assert_eq!(reason, expected, "vector {}", id);
            }
            _ => panic!("vector {} produced unexpected error: {:?}", id, err),
        }
    }
}
```

- [ ] **Step 2: Swift parity test**

`bindings/swift/Tests/JovaCoreTests/BtcVectorsTests.swift`: mirror the Rust file. Decoder uses `decodeChain` and a new `decodeUnsignedBitcoin` / `decodeMessageBitcoin` helper appended to `VectorDecoders.swift` (Phase 1 file). The vector iteration shape is identical.

(Full file follows the pattern from `EvmVectorsTests.swift` — copy-modify, change the chain filters from EVM to bitcoin.)

- [ ] **Step 3: Kotlin parity test**

`bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/BtcVectorsTest.kt`: same shape as `EvmVectorsTest.kt`, change filters and decoders.

- [ ] **Step 4: Run the full vector parity**

```bash
just test                                                         # Rust + verify-spec
just build-ios && (cd bindings/swift && swift test)               # macOS only
just build-android && (cd bindings/kotlin && ./gradlew :jova-core:test)
```

Expected: every BTC vector passes byte-identically on every binding.

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core/tests/vectors_btc.rs bindings/
git commit -m "test: BTC vector parity Rust + Swift + Kotlin"
```

---

## Task 9: Property tests + fuzz targets

**Files:**
- Create: `crates/jova-core/tests/properties_btc.rs`
- Create: `fuzz/fuzz_targets/fuzz_psbt_sign.rs`
- Create: `fuzz/fuzz_targets/fuzz_btc_address_parse.rs`
- Create: `fuzz/fuzz_targets/fuzz_bip322_verify.rs`

- [ ] **Step 1: Property tests**

`crates/jova-core/tests/properties_btc.rs`:

```rust
use jova_core::*;
use proptest::prelude::*;

const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

proptest! {
    #[test]
    fn btc_address_is_deterministic(_n in 0u32..1000) {
        let w1 = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let w2 = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let a1 = w1.address(&JovaChain::Bitcoin, 0).unwrap();
        let a2 = w2.address(&JovaChain::Bitcoin, 0).unwrap();
        prop_assert_eq!(a1.value, a2.value);
    }

    #[test]
    fn btc_address_validates(_n in 0u32..100) {
        let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let a = w.address(&JovaChain::Bitcoin, 0).unwrap();
        prop_assert!(is_valid_address(&a.value, &JovaChain::Bitcoin));
    }

    #[test]
    fn random_strings_do_not_validate(s in "\\PC*") {
        // Almost all random strings are not valid bech32 P2WPKH addresses.
        // We don't assert false (a tiny fraction by luck might validate); we
        // assert the validator doesn't panic.
        let _ = is_valid_address(&s, &JovaChain::Bitcoin);
    }
}
```

- [ ] **Step 2: Fuzz targets**

Add three new entries to `fuzz/Cargo.toml`'s `[[bin]]` blocks: `fuzz_psbt_sign`, `fuzz_btc_address_parse`, `fuzz_bip322_verify`.

`fuzz/fuzz_targets/fuzz_psbt_sign.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use jova_core::{JovaChain, JovaWallet, UnsignedTx};

const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    let _ = JovaChain::Bitcoin;
    let psbt_b64 = B64.encode(data);
    let unsigned = UnsignedTx::Bitcoin { psbt_base64: psbt_b64 };
    let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
    let _ = w.sign_tx(&unsigned);
});
```

`fuzz/fuzz_targets/fuzz_btc_address_parse.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{is_valid_address, JovaChain};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = is_valid_address(s, &JovaChain::Bitcoin);
    }
});
```

`fuzz/fuzz_targets/fuzz_bip322_verify.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{JovaChain, JovaWallet, SignableMessage, BtcMsgScheme};

const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    if let Ok(message) = std::str::from_utf8(data) {
        let msg = SignableMessage::Bitcoin {
            message: message.to_string(),
            address: "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".into(),
            scheme: BtcMsgScheme::Bip322,
        };
        let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let _ = w.sign_message(&msg);
    }
});
```

- [ ] **Step 3: Run each fuzzer for 60 seconds locally**

```bash
just fuzz   # runs all targets via the justfile recipe
```

Expected: no crashes.

- [ ] **Step 4: Commit**

```bash
git add crates/jova-core/tests/properties_btc.rs fuzz/
git commit -m "test: property tests + fuzz targets for BTC"
```

---

## Task 10: Android migration spot-check

**Files:**
- Create: `tools/btc-migration-check/Cargo.toml`
- Create: `tools/btc-migration-check/src/main.rs`
- Create: `docs/btc-migration-check.md`

- [ ] **Step 1: The check tool**

The Android app team has provided `tools/btc-migration-check/known-android-mappings.csv` with 100+ rows of `mnemonic,address` pairs from production storage.

`tools/btc-migration-check/Cargo.toml`:

```toml
[package]
name = "jova-btc-migration-check"
version = "0.0.0"
edition.workspace = true
publish = false

[[bin]]
name = "btc-migration-check"
path = "src/main.rs"

[dependencies]
jova-core.workspace = true
csv = "1.3"
```

`tools/btc-migration-check/src/main.rs`:

```rust
use jova_core::{JovaChain, JovaWallet};
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = "tools/btc-migration-check/known-android-mappings.csv";
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .expect("csv readable");

    let mut total = 0usize;
    let mut matches = 0usize;
    let mut mismatches: Vec<(String, String, String)> = Vec::new();

    for record in rdr.records() {
        let r = record.expect("record");
        let mnemonic = &r[0];
        let expected_addr = &r[1];

        let wallet = JovaWallet::from_mnemonic(mnemonic, "")
            .expect("legacy mnemonic should be valid");
        let derived = wallet.address(&JovaChain::Bitcoin, 0)
            .expect("BTC derivation should succeed");

        total += 1;
        if derived.value == expected_addr {
            matches += 1;
        } else {
            mismatches.push((mnemonic.into(), expected_addr.into(), derived.value));
        }
    }

    println!("BTC migration check: {}/{} match", matches, total);
    if !mismatches.is_empty() {
        for (m, expected, got) in &mismatches {
            // Do NOT log the full mnemonic — first two words only, as a debug aid.
            let m_short: String = m.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            eprintln!("MISMATCH: '{}...' expected={} got={}", m_short, expected, got);
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
```

- [ ] **Step 2: Run the check**

```bash
cargo run -p jova-btc-migration-check
```

Expected: `BTC migration check: N/N match` (where N is the number of rows in the CSV; should be ≥100).

If any mismatch: STOP. The discrepancy means the SDK and the legacy Android app derive different addresses from the same seed — a user's funds would land at the wrong address. Investigation required before proceeding.

Common causes:
- Legacy app uses a different derivation path. (Verify: did legacy use `m/84'/0'/0'/0/0` or a different account/index?)
- Legacy app applies a passphrase. (Verify with the app team.)
- Bech32 encoding edge case (network prefix, witness version).

- [ ] **Step 3: Document the result**

Create `docs/btc-migration-check.md`:

```markdown
# BTC Migration Spot-Check Report

**Date:** YYYY-MM-DD
**SDK version:** v0.2.0-rc

## Inputs
- `tools/btc-migration-check/known-android-mappings.csv` — N rows
- Source: Android app team production export

## Result
N/N match → migration safe.

## Procedure
`cargo run -p jova-btc-migration-check` against the CSV.
The tool derives `m/84'/0'/0'/0/0` from each mnemonic, computes the
P2WPKH bech32 address, compares to the legacy stored value.

## Out-of-scope
- Mnemonics with passphrases (none in production at the time of export).
- Account indices > 0 (apps don't use them).
```

- [ ] **Step 4: Commit**

```bash
git add tools/btc-migration-check/ docs/btc-migration-check.md
git commit -m "tools: BTC migration spot-check — N/N match against legacy Android storage"
```

(The CSV file itself is **NOT** committed — it contains user mnemonics. Document this explicitly: `tools/btc-migration-check/.gitignore` includes `known-android-mappings.csv`.)

---

## Task 11: Mainnet smoke test

**Files:**
- Create: `docs/btc-mainnet-smoke.md`

- [ ] **Step 1: Engineer-driven manual test**

Engineer driving Phase 2:

1. Sets up a real Bitcoin mainnet wallet with the SDK using a fresh mnemonic — generates 12 words via `JovaWallet::createMnemonic(.bits128)`, derives a BIP-84 address via `wallet.address(.bitcoin)`, deposits a small amount (recommend 10,000 sats from an exchange withdrawal).
2. Constructs a PSBT (via the backend's PSBT builder, OR via `bdk-cli` against the same mainnet UTXO set if backend isn't ready) sending most of those sats to a destination address with a small fee.
3. Calls `wallet.signTx(unsigned)` with `UnsignedTx::Bitcoin { psbt_base64: ... }`.
4. Broadcasts the resulting `signed.raw_hex` (must not have the `psbt:` prefix — full single-party flow).
5. Watches mempool/blockchain for confirmation. Document the tx hash.

- [ ] **Step 2: Document the result**

`docs/btc-mainnet-smoke.md`:

```markdown
# BTC Mainnet Smoke Test

**Date:** YYYY-MM-DD
**SDK version:** v0.2.0-rc

## Tx hash
<paste>

## Block confirmed at
<paste>

## Procedure
1. Generated fresh mnemonic via SDK.
2. Funded the BIP-84 address with 10,000 sats.
3. Constructed PSBT spending all 10,000 sats minus fee to a destination.
4. Signed via SDK; raw hex broadcast via mempool.space.
5. Confirmed in 10–30 minutes.
```

- [ ] **Step 3: Commit**

```bash
git add docs/btc-mainnet-smoke.md
git commit -m "docs: BTC mainnet smoke — tx <hash> confirmed"
```

---

## Task 12: Open PR, CI green, tag v0.2.0

- [ ] **Step 1: Push branch + open PR**

```bash
git push -u origin feat/phase-2-bitcoin
gh pr create --title "Phase 2: Bitcoin (BIP-84 + PSBT + BIP-322)" --body "$(cat <<'EOF'
## Summary
- BIP-84 derivation in jova-core-primitives
- P2WPKH (bech32) address derivation in jova-core-chains::btc
- PSBT signing: single-input, multi-input, multi-party
- BIP-322 message signing + legacy signMessage fallback
- 12 BTC vectors (3 address × 2 mnemonics + 3 sign_tx + 2 sign_message + 3 error)
- Vector parity passing on Rust + Swift + Kotlin
- Property tests + 3 fuzz targets
- 100/100 migration spot-check vs legacy Android storage
- Mainnet smoke confirmed: tx <hash>

## Test plan
- [x] cargo test --workspace passes
- [x] All CI workflows pass
- [x] cargo fuzz run for 60s on each new target — no crashes
- [x] Migration spot-check 100/100 match
- [x] Mainnet smoke tx confirmed

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 2: After CI green and review approval, merge**

```bash
gh pr merge --squash --delete-branch
git checkout main && git pull
```

- [ ] **Step 3: Tag v0.2.0**

```bash
git tag -a v0.2.0 -m "v0.2.0 — Phase 2 Bitcoin"
git push origin v0.2.0
```

---

## Self-review

- [ ] Every task has exact paths and exact commands.
- [ ] Every code block has the actual code (no `// implement here`).
- [ ] BIP-84 derivation tested against the official BIP-84 vectors.
- [ ] PSBT signing tested for single-input, multi-input owned, and multi-party scenarios — all from real `bdk-cli` captures.
- [ ] BIP-322 tested against `bdk-cli` reference output.
- [ ] Address validation rejects legacy P2PKH and Taproot in v1 (only P2WPKH).
- [ ] Multi-party PSBT result is signaled by `psbt:` prefix in `raw_hex` — documented in integration docs.
- [ ] `tools/verify-spec` rejects placeholder strings in the new vectors.
- [ ] Migration spot-check is gated on CSV from app team; commit policy excludes the CSV from git.
- [ ] Mainnet smoke is engineer-driven, documented, with a real tx hash.

---

## What this plan does NOT do

- Does not implement Taproot (BIP-86). Future `JovaChain.bitcoinTaproot` variant.
- Does not implement multi-sig descriptor flows. v1 is single-sig BIP-84.
- Does not run WASM functional tests for BTC. Phase 6 covers that.
- Does not introduce a "broadcast" function. SDK signs; backend broadcasts.

---

## Estimated time

3–4 weeks for a senior team. Time sinks:
1. Capture script reliability — bdk-cli + regtest setup is the bulk.
2. Multi-party PSBT semantics — easy to get edge cases wrong; rely on the captures.
3. Migration spot-check coordination — depends on Android team export.
4. Mainnet smoke — paced by network confirmation times.
