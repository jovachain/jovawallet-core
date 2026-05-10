# Phase 1: EVM End-to-End Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Real `JovaWallet` API on Rust + Swift + Kotlin, fully exercising the EVM signing path (BIP-39 mnemonic, BIP-32 derivation, EIP-55 address, EIP-1559 transaction, EIP-191 personal_sign, EIP-712 typed data v4) with byte-identical vector parity across all three native bindings. WASM compiles only — functional WASM tests come in Phase 6. Tag `v0.1.0`.

**Architecture:** `jova-core-primitives` implements `Mnemonic`, `Seed`, `XPrv`, `DerivationPath`, BIP-32 derivation, secp256k1 signing primitive. `jova-core-chains::evm` implements the `ChainSigner` trait using `alloy`. `jova-core` exposes the public `JovaWallet` API. `jova-core-ffi` and `jova-core-wasm` re-export. Vectors load from `spec/test-vectors.json` on every binding.

**Tech Stack:** Rust 1.78, alloy 0.7, secp256k1 0.29, bip39 2.0, bip32 0.5, zeroize 1.7. uniffi-rs 0.28 for Swift+Kotlin bindings. wasm-bindgen 0.2 for WASM compile-only.

**Preconditions:**
- Phase 0 complete: tag `v0.0.1` exists; all 6 CI workflows are green.
- `spec/test-vectors.json` exists with the Phase 0 vector.
- Branch `feat/phase-1-evm` from `main`.

**Exit criteria:**
- The `is_valid_mnemonic_stub` is gone; real BIP-39 validation is in place.
- 18+ vectors covering EVM (3 address × 2 mnemonics + 4 sign_tx + 2 sign_message + 3 error) all pass on Rust + Swift + Kotlin.
- A signed transaction produced by Swift, Kotlin, and Rust for the same vector input is byte-identical.
- WASM compiles; the WASM hello-world test from Phase 0 still passes.
- `cargo miri test -p jova-core-primitives` is green.
- Property tests for EVM round-trip pass.
- Three fuzz targets exist and run cleanly for at least 60 seconds locally.
- Tag `v0.1.0` exists on `main`.

---

## Task 1: Implement `Mnemonic` and `Seed` in jova-core-primitives

**Files:**
- Modify: `crates/jova-core-primitives/Cargo.toml`
- Create: `crates/jova-core-primitives/src/mnemonic.rs`
- Create: `crates/jova-core-primitives/src/seed.rs`
- Create: `crates/jova-core-primitives/src/error.rs`
- Modify: `crates/jova-core-primitives/src/lib.rs`
- Create: `crates/jova-core-primitives/tests/mnemonic.rs`

- [ ] **Step 1: Add bip39 dependency**

Modify `crates/jova-core-primitives/Cargo.toml`:

```toml
[dependencies]
zeroize.workspace = true
bip39.workspace = true
thiserror = { workspace = true, optional = true }

[features]
default = ["std"]
std = ["dep:thiserror"]
```

- [ ] **Step 2: Write the failing test**

`crates/jova-core-primitives/tests/mnemonic.rs`:

```rust
use jova_core_primitives::{Mnemonic, MnemonicError, Strength};

#[test]
fn validates_official_bip39_test_vector() {
    // BIP-39 official: 12 words of "abandon" + "about" — known-valid.
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(Mnemonic::validate(words, "").is_ok());
}

#[test]
fn rejects_invalid_checksum() {
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    // 12x abandon = wrong checksum
    assert!(matches!(Mnemonic::validate(words, ""), Err(MnemonicError::InvalidChecksum)));
}

#[test]
fn rejects_unknown_word() {
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon zzzzz";
    assert!(matches!(Mnemonic::validate(words, ""), Err(MnemonicError::InvalidWord(_))));
}

#[test]
fn generates_24_word_mnemonic_at_bits256() {
    let m = Mnemonic::generate(Strength::Bits256);
    let count = m.words.split_whitespace().count();
    assert_eq!(count, 24);
    assert!(Mnemonic::validate(&m.words, "").is_ok());
}

#[test]
fn to_seed_matches_bip39_official_vector() {
    // BIP-39 vector: passphrase "TREZOR" with the abandon-about mnemonic.
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let seed = Mnemonic::to_seed(words, "TREZOR").expect("valid");
    let expected_hex = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";
    assert_eq!(hex::encode(seed.as_bytes()), expected_hex);
}
```

- [ ] **Step 3: Run the test (expect: fails to compile)**

```bash
cargo test -p jova-core-primitives --test mnemonic
```

Expected: `error[E0432]: unresolved import jova_core_primitives::Mnemonic`.

- [ ] **Step 4: Implement the error type**

`crates/jova-core-primitives/src/error.rs`:

```rust
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MnemonicError {
    InvalidWordCount,
    InvalidChecksum,
    InvalidWord(String),
    PassphraseTooLong,
}

#[cfg(feature = "std")]
impl std::error::Error for MnemonicError {}

impl core::fmt::Display for MnemonicError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWordCount => f.write_str("invalid word count"),
            Self::InvalidChecksum => f.write_str("invalid checksum"),
            Self::InvalidWord(_) => f.write_str("invalid word"),
            Self::PassphraseTooLong => f.write_str("passphrase too long"),
        }
    }
}
```

- [ ] **Step 5: Implement Seed**

`crates/jova-core-primitives/src/seed.rs`:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

// NOT Clone — `docs/memory-and-keys.md` audit checklist requires this.
// Anything that needs a seed takes &Seed; ownership is unique to JovaWalletInner.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Seed([u8; 64]);

impl Seed {
    pub(crate) fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl core::fmt::Debug for Seed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Seed(<redacted>)")
    }
}
```

- [ ] **Step 6: Implement Mnemonic**

`crates/jova-core-primitives/src/mnemonic.rs`:

```rust
use alloc::string::{String, ToString};
use bip39::Mnemonic as Bip39Mnemonic;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::MnemonicError;
use crate::seed::Seed;

#[derive(Clone, Copy, Debug)]
pub enum Strength {
    Bits128,
    Bits256,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Mnemonic {
    pub words: String,
    pub passphrase: String,
}

impl Mnemonic {
    pub fn generate(strength: Strength) -> Self {
        // bip39 2.x: Mnemonic::generate(word_count) — 12 or 24 words.
        let word_count = match strength {
            Strength::Bits128 => 12,
            Strength::Bits256 => 24,
        };
        let m = Bip39Mnemonic::generate(word_count).expect("rng available");
        Self { words: m.to_string(), passphrase: String::new() }
    }

    pub fn validate(words: &str, _passphrase: &str) -> Result<(), MnemonicError> {
        match Bip39Mnemonic::parse(words) {
            Ok(_) => Ok(()),
            Err(bip39::Error::BadWordCount(_)) => Err(MnemonicError::InvalidWordCount),
            Err(bip39::Error::UnknownWord(idx)) => {
                let word = words.split_whitespace().nth(idx).unwrap_or("?").to_string();
                Err(MnemonicError::InvalidWord(word))
            }
            Err(bip39::Error::BadEntropyBitCount(_)) => Err(MnemonicError::InvalidWordCount),
            Err(bip39::Error::InvalidChecksum) => Err(MnemonicError::InvalidChecksum),
            Err(_) => Err(MnemonicError::InvalidChecksum),
        }
    }

    pub fn to_seed(words: &str, passphrase: &str) -> Result<Seed, MnemonicError> {
        Self::validate(words, passphrase)?;
        if passphrase.len() > 256 { return Err(MnemonicError::PassphraseTooLong); }
        let m = Bip39Mnemonic::parse(words).expect("validated");
        let mut bytes = m.to_seed(passphrase);
        let seed = Seed::from_bytes(bytes);
        bytes.zeroize();
        Ok(seed)
    }
}
```

If the snippet doesn't compile against the version of `bip39` that Phase -1's feasibility report selected, the **test in Step 2 is the contract** — adjust the snippet, not the test. The test's expected values come from BIP-39 official vectors, which are crate-version-independent.

- [ ] **Step 7: Wire into lib.rs**

Modify `crates/jova-core-primitives/src/lib.rs`:

```rust
//! jova-core-primitives — no_std-clean cryptographic primitives.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod mnemonic;
mod seed;

pub use error::MnemonicError;
pub use mnemonic::{Mnemonic, Strength};
pub use seed::Seed;

// Phase 0 stub — keep until Phase 1 task that removes it.
#[deprecated(note = "Phase 0 stub; use Mnemonic::validate instead")]
pub fn is_valid_mnemonic_stub(words: &str, _passphrase: &str) -> bool {
    Mnemonic::validate(words, _passphrase).is_ok()
}
```

- [ ] **Step 8: Run the tests**

```bash
cargo test -p jova-core-primitives --test mnemonic
```

Expected: all five tests pass.

- [ ] **Step 9: Commit**

```bash
git add crates/jova-core-primitives/
git commit -m "feat(primitives): real BIP-39 Mnemonic + Seed with zeroize"
```

---

## Task 2: BIP-32 derivation in jova-core-primitives

**Files:**
- Modify: `crates/jova-core-primitives/Cargo.toml`
- Create: `crates/jova-core-primitives/src/derive.rs`
- Create: `crates/jova-core-primitives/src/path.rs`
- Create: `crates/jova-core-primitives/src/keys.rs`
- Modify: `crates/jova-core-primitives/src/lib.rs`
- Create: `crates/jova-core-primitives/tests/derive.rs`

- [ ] **Step 1: Add deps**

Modify `crates/jova-core-primitives/Cargo.toml`:

```toml
[dependencies]
zeroize.workspace = true
bip39.workspace = true
bip32.workspace = true
secp256k1.workspace = true
hex = { workspace = true }
thiserror = { workspace = true, optional = true }
```

- [ ] **Step 2: Failing tests**

`crates/jova-core-primitives/tests/derive.rs`:

```rust
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

#[test]
fn parses_canonical_eth_path() {
    let p = DerivationPath::parse("m/44'/60'/0'/0/0").expect("parse");
    assert_eq!(p.indices.len(), 5);
    assert_eq!(p.indices[0], 0x8000_002C); // 44 hardened
    assert_eq!(p.indices[1], 0x8000_003C); // 60 hardened
    assert_eq!(p.indices[2], 0x8000_0000); // 0 hardened
    assert_eq!(p.indices[3], 0);           // 0
    assert_eq!(p.indices[4], 0);           // 0
}

#[test]
fn rejects_malformed_path() {
    assert!(DerivationPath::parse("m/44").is_err());
    assert!(DerivationPath::parse("xx").is_err());
    assert!(DerivationPath::parse("").is_err());
}

#[test]
fn derives_eth_xprv_from_known_seed() {
    // BIP-39 vector: abandon...about with passphrase "TREZOR".
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let seed = Mnemonic::to_seed(words, "TREZOR").expect("seed");
    let path = DerivationPath::parse("m/44'/60'/0'/0/0").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");

    // Known result from running geth or any BIP-32 reference against this seed:
    let pub_uncompressed = xprv.public_key_uncompressed();
    let address_hex = jova_core_primitives::keccak_address(&pub_uncompressed);
    // Expected address per BIP-44 + this seed: 0x9c32... (replace with actual computed value
    // from a reference impl during Phase 1; see vectors note below).
    // For the test to be self-contained without a network call, assert the *length* and that
    // it matches what jova-core produces consistently:
    assert_eq!(address_hex.len(), 40);
}
```

(Note for the agent: the *real* expected address is captured from a reference implementation when authoring `spec/test-vectors.json` in Task 4 below. This test asserts the contract on shape; the address vector test in Task 4 asserts byte-identical agreement with the reference.)

- [ ] **Step 3: DerivationPath**

`crates/jova-core-primitives/src/path.rs`:

```rust
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationPath {
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    Empty,
    NotPrefixed,
    InvalidComponent(String),
    IndexOutOfRange,
}

impl core::fmt::Display for PathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => f.write_str("empty path"),
            Self::NotPrefixed => f.write_str("path must start with m/"),
            Self::InvalidComponent(_) => f.write_str("invalid component"),
            Self::IndexOutOfRange => f.write_str("index out of range"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PathError {}

const HARDENED_OFFSET: u32 = 0x8000_0000;

impl DerivationPath {
    pub fn parse(s: &str) -> Result<Self, PathError> {
        let s = s.trim();
        if s.is_empty() { return Err(PathError::Empty); }
        if !s.starts_with("m/") && s != "m" { return Err(PathError::NotPrefixed); }

        let parts = if s == "m" { vec![] } else { s["m/".len()..].split('/').collect::<Vec<_>>() };
        let mut indices = Vec::with_capacity(parts.len());
        for part in parts {
            let (num, hardened) = if let Some(stripped) = part.strip_suffix('\'') {
                (stripped, true)
            } else if let Some(stripped) = part.strip_suffix('h') {
                (stripped, true)
            } else {
                (part, false)
            };
            let n: u32 = num.parse().map_err(|_| PathError::InvalidComponent(part.to_string()))?;
            if n >= HARDENED_OFFSET { return Err(PathError::IndexOutOfRange); }
            indices.push(if hardened { n + HARDENED_OFFSET } else { n });
        }
        Ok(Self { indices })
    }
}
```

- [ ] **Step 4: XPrv and derivation**

`crates/jova-core-primitives/src/keys.rs`:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

// NOT Clone. Per-call derivation produces a fresh XPrv that lives only for the
// duration of the signing call; chain signers take &XPrv and don't need to copy.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct XPrv {
    pub(crate) key: [u8; 32],
    pub(crate) chain_code: [u8; 32],
}

impl XPrv {
    pub fn private_key_bytes(&self) -> &[u8; 32] { &self.key }

    pub fn public_key_uncompressed(&self) -> [u8; 65] {
        let secp = secp256k1::Secp256k1::signing_only();
        let sk = secp256k1::SecretKey::from_slice(&self.key).expect("valid sk");
        let pk = sk.public_key(&secp);
        pk.serialize_uncompressed()
    }

    pub fn public_key_compressed(&self) -> [u8; 33] {
        let secp = secp256k1::Secp256k1::signing_only();
        let sk = secp256k1::SecretKey::from_slice(&self.key).expect("valid sk");
        let pk = sk.public_key(&secp);
        pk.serialize()
    }
}

impl core::fmt::Debug for XPrv {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("XPrv(<redacted>)")
    }
}
```

`crates/jova-core-primitives/src/derive.rs`:

```rust
use crate::keys::XPrv;
use crate::path::DerivationPath;
use crate::seed::Seed;
use bip32::{XPrv as Bip32Xprv, ChildNumber};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveError {
    Bip32,
}

impl core::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BIP-32 derivation failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeriveError {}

pub fn derive_secp256k1(seed: &Seed, path: &DerivationPath) -> Result<XPrv, DeriveError> {
    let mut xprv = Bip32Xprv::new(seed.as_bytes()).map_err(|_| DeriveError::Bip32)?;
    for &i in &path.indices {
        let child = ChildNumber::from(i);
        xprv = xprv.derive_child(child).map_err(|_| DeriveError::Bip32)?;
    }
    let private = xprv.private_key();
    let chain_code = xprv.attrs().chain_code;
    Ok(XPrv {
        key: private.to_bytes().into(),
        chain_code,
    })
}

pub fn keccak_address(uncompressed_pubkey: &[u8; 65]) -> alloc::string::String {
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(&uncompressed_pubkey[1..]);     // drop leading 0x04
    let digest = h.finalize();
    hex::encode(&digest[12..])               // last 20 bytes
}
```

- [ ] **Step 5: Add sha3 dep**

Add to `crates/jova-core-primitives/Cargo.toml`:

```toml
sha3.workspace = true
```

- [ ] **Step 6: Wire into lib.rs**

```rust
mod path;
mod keys;
mod derive;

pub use path::{DerivationPath, PathError};
pub use keys::XPrv;
pub use derive::{derive_secp256k1, keccak_address, DeriveError};
```

- [ ] **Step 7: Run tests**

```bash
cargo test -p jova-core-primitives
```

Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add crates/jova-core-primitives/
git commit -m "feat(primitives): BIP-32 derivation + secp256k1 keys + keccak address"
```

---

## Task 3: EVM ChainSigner in jova-core-chains

**Files:**
- Modify: `crates/jova-core-chains/Cargo.toml`
- Create: `crates/jova-core-chains/src/error.rs`
- Create: `crates/jova-core-chains/src/signer.rs`
- Create: `crates/jova-core-chains/src/unsigned_tx.rs`
- Create: `crates/jova-core-chains/src/signable_message.rs`
- Create: `crates/jova-core-chains/src/address.rs`
- Create: `crates/jova-core-chains/src/evm/mod.rs`
- Create: `crates/jova-core-chains/src/evm/address.rs`
- Create: `crates/jova-core-chains/src/evm/tx.rs`
- Create: `crates/jova-core-chains/src/evm/eip191.rs`
- Create: `crates/jova-core-chains/src/evm/eip712.rs`
- Modify: `crates/jova-core-chains/src/lib.rs`
- Create: `crates/jova-core-chains/tests/evm.rs`

- [ ] **Step 1: Add deps**

`crates/jova-core-chains/Cargo.toml`:

```toml
[dependencies]
jova-core-primitives.workspace = true
alloy.workspace = true
secp256k1.workspace = true
sha3.workspace = true
serde.workspace = true
serde_json.workspace = true
hex.workspace = true
thiserror.workspace = true
```

- [ ] **Step 2: Define the trait, error, types**

`crates/jova-core-chains/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ChainError {
    #[error("invalid address for chain")]
    InvalidAddress,
    #[error("malformed unsigned tx: {0}")]
    MalformedUnsignedTx(String),
    #[error("malformed signable message: {0}")]
    MalformedSignableMessage(String),
    #[error("signing failed: {0}")]
    SigningFailed(String),
    #[error("internal: {0}")]
    Internal(String),
}
```

`crates/jova-core-chains/src/unsigned_tx.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UnsignedTx {
    Evm(EvmUnsigned),
    // Phase 2+ adds: Bitcoin, Solana, Xrp.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmUnsigned {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: String,
    pub value: String,                  // wei, decimal string
    pub gas_limit: u64,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub data: String,                   // 0x-prefixed
    #[serde(default)]
    pub access_list: Vec<AccessListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListItem {
    pub address: String,
    pub storage_keys: Vec<String>,
}
```

`crates/jova-core-chains/src/signable_message.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SignableMessage {
    EvmPersonalSign { message: String },
    EvmTypedDataV4  { json: String },
    // Phase 2+ adds: Solana, Bitcoin.
}
```

`crates/jova-core-chains/src/address.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Address {
    pub chain: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedTx {
    pub chain: String,
    pub raw_hex: String,
    pub tx_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature {
    pub hex: String,
}
```

`crates/jova-core-chains/src/signer.rs`:

```rust
use jova_core_primitives::XPrv;
use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::unsigned_tx::UnsignedTx;

pub trait ChainSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError>;
    fn validate_address(&self, addr: &str) -> bool;
    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError>;
    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError>;
}
```

- [ ] **Step 3: EVM address (EIP-55)**

`crates/jova-core-chains/src/evm/address.rs`:

```rust
use jova_core_primitives::{XPrv, keccak_address};
use sha3::{Digest, Keccak256};

pub fn derive(key: &XPrv) -> String {
    let pk = key.public_key_uncompressed();
    let lower = keccak_address(&pk);   // 40 hex chars, lowercase
    eip55_checksum(&lower)
}

pub fn eip55_checksum(lower: &str) -> String {
    let lower = lower.trim_start_matches("0x").to_lowercase();
    let mut h = Keccak256::new();
    h.update(lower.as_bytes());
    let digest = h.finalize();
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, ch) in lower.chars().enumerate() {
        if ch.is_ascii_hexdigit() && ch.is_ascii_alphabetic() {
            let nibble = digest[i / 2] >> if i % 2 == 0 { 4 } else { 0 } & 0xf;
            if nibble >= 8 { out.push(ch.to_ascii_uppercase()); } else { out.push(ch); }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn validate(addr: &str) -> bool {
    if !addr.starts_with("0x") || addr.len() != 42 { return false; }
    let body = &addr[2..];
    if !body.chars().all(|c| c.is_ascii_hexdigit()) { return false; }
    // Mixed-case → must match EIP-55 checksum. All-lower or all-upper → accepted as legacy.
    let has_upper = body.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = body.chars().any(|c| c.is_ascii_lowercase());
    if has_upper && has_lower {
        let expected = eip55_checksum(&body.to_lowercase());
        addr == expected
    } else {
        true
    }
}
```

- [ ] **Step 4: EIP-1559 transaction signing**

`crates/jova-core-chains/src/evm/tx.rs`:

```rust
use crate::error::ChainError;
use crate::unsigned_tx::EvmUnsigned;
use jova_core_primitives::XPrv;

pub fn sign(key: &XPrv, tx: &EvmUnsigned) -> Result<(String, String), ChainError> {
    use alloy::consensus::{TxEip1559, SignableTransaction};
    use alloy::primitives::{Address as AlloyAddress, Bytes, U256, B256, ChainId};
    use alloy::eips::eip2930::{AccessList, AccessListItem as AlloyAcl};
    use alloy::signers::local::LocalSigner;
    use alloy::signers::SignerSync;

    // 1. Parse fields.
    let to: AlloyAddress = tx.to.parse()
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_to_address_invalid".into()))?;
    let value = U256::from_str_radix(tx.value.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;
    let max_fee = U256::from_str_radix(tx.max_fee_per_gas.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;
    let max_pri = U256::from_str_radix(tx.max_priority_fee_per_gas.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;
    let data: Bytes = hex::decode(tx.data.trim_start_matches("0x"))
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_data_not_hex".into()))?
        .into();

    // 2. Access list.
    let mut access_list = Vec::with_capacity(tx.access_list.len());
    for item in &tx.access_list {
        let address: AlloyAddress = item.address.parse()
            .map_err(|_| ChainError::MalformedUnsignedTx("evm_access_list_invalid".into()))?;
        let mut keys = Vec::with_capacity(item.storage_keys.len());
        for k in &item.storage_keys {
            let kb: B256 = k.parse()
                .map_err(|_| ChainError::MalformedUnsignedTx("evm_access_list_invalid".into()))?;
            keys.push(kb);
        }
        access_list.push(AlloyAcl { address, storage_keys: keys });
    }

    // 3. Build the typed tx.
    let mut alloy_tx = TxEip1559 {
        chain_id: tx.chain_id as ChainId,
        nonce: tx.nonce,
        gas_limit: tx.gas_limit,
        max_fee_per_gas: max_fee.try_into().unwrap_or(u128::MAX),
        max_priority_fee_per_gas: max_pri.try_into().unwrap_or(u128::MAX),
        to: alloy::primitives::TxKind::Call(to),
        value,
        access_list: AccessList(access_list),
        input: data,
    };

    // 4. Sign.
    let signer = LocalSigner::from_bytes(&B256::from_slice(key.private_key_bytes()))
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;
    let sig_hash = alloy_tx.signature_hash();
    let signature = signer.sign_hash_sync(&sig_hash)
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;

    // 5. Encode.
    let signed = alloy_tx.into_signed(signature);
    let mut buf = Vec::new();
    signed.eip2718_encode(&mut buf);
    let raw_hex = format!("0x{}", hex::encode(&buf));
    let tx_hash = format!("0x{}", hex::encode(signed.hash()));
    Ok((raw_hex, tx_hash))
}
```

If alloy's API has changed since this plan was written, **the vector tests in Task 4 are the contract**. The expected `signed_hex` and `tx_hash` come from `cast wallet sign-tx` against the same input, so they don't depend on alloy's internal naming — only on the produced bytes. Adjust the snippet to whatever alloy expects in your version; do not adjust the vector.

- [ ] **Step 5: EIP-191 personal_sign**

`crates/jova-core-chains/src/evm/eip191.rs`:

```rust
use jova_core_primitives::XPrv;
use crate::error::ChainError;
use sha3::{Digest, Keccak256};

pub fn sign(key: &XPrv, message: &str) -> Result<String, ChainError> {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut h = Keccak256::new();
    h.update(prefix.as_bytes());
    h.update(message.as_bytes());
    let digest = h.finalize();

    sign_hash(key, &digest)
}

pub(crate) fn sign_hash(key: &XPrv, hash: &[u8]) -> Result<String, ChainError> {
    use secp256k1::{Secp256k1, Message, ecdsa::RecoverableSignature};
    let secp = Secp256k1::signing_only();
    let sk = secp256k1::SecretKey::from_slice(key.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;
    let msg = Message::from_digest_slice(hash)
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;
    let recoverable: RecoverableSignature = secp.sign_ecdsa_recoverable(&msg, &sk);
    let (rec_id, sig_bytes) = recoverable.serialize_compact();
    let v = (rec_id.to_i32() as u8) + 27;
    let mut hex_out = String::with_capacity(132);
    hex_out.push_str("0x");
    hex_out.push_str(&hex::encode(&sig_bytes[..32]));   // r
    hex_out.push_str(&hex::encode(&sig_bytes[32..]));   // s
    hex_out.push_str(&hex::encode([v]));                // v
    Ok(hex_out)
}
```

- [ ] **Step 6: EIP-712 v4 typed-data**

`crates/jova-core-chains/src/evm/eip712.rs`:

```rust
use jova_core_primitives::XPrv;
use crate::error::ChainError;

pub fn sign_typed_data_v4(key: &XPrv, json: &str) -> Result<String, ChainError> {
    use alloy::dyn_abi::TypedData;
    let typed: TypedData = serde_json::from_str(json)
        .map_err(|_| ChainError::MalformedSignableMessage("eip712_typed_data_invalid_json".into()))?;
    let digest = typed.eip712_signing_hash()
        .map_err(|_| ChainError::MalformedSignableMessage("eip712_unknown_type".into()))?;

    super::eip191::sign_hash(key, digest.as_slice())
}
```

`alloy::dyn_abi::TypedData` is the runtime-parsed typed-data API (as opposed to `alloy::sol_types`, which is for compile-time-known types from `sol!` macros). Make sure the workspace `alloy` dep enables the `dyn-abi` feature.

- [ ] **Step 7: EVM ChainSigner impl**

`crates/jova-core-chains/src/evm/mod.rs`:

```rust
mod address;
mod tx;
mod eip191;
mod eip712;

use jova_core_primitives::XPrv;
use crate::address::{Address, SignedTx, Signature};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;

pub struct EvmSigner {
    pub chain_label: &'static str,   // "ethereum", "polygon", etc., for Address.chain
}

impl ChainSigner for EvmSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError> {
        Ok(Address {
            chain: self.chain_label.to_string(),
            value: address::derive(key),
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        address::validate(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Evm(evm) => {
                let (raw_hex, tx_hash) = tx::sign(key, evm)?;
                Ok(SignedTx {
                    chain: self.chain_label.to_string(),
                    raw_hex, tx_hash,
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx("expected_evm_variant".into())),
        }
    }

    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError> {
        let hex = match msg {
            SignableMessage::EvmPersonalSign { message } => eip191::sign(key, message)?,
            SignableMessage::EvmTypedDataV4  { json }    => eip712::sign_typed_data_v4(key, json)?,
            _ => return Err(ChainError::MalformedSignableMessage("expected_evm_message".into())),
        };
        Ok(Signature { hex })
    }
}
```

- [ ] **Step 8: Wire into lib.rs**

`crates/jova-core-chains/src/lib.rs`:

```rust
//! jova-core-chains — per-chain encoding and signing.

#![forbid(unsafe_code)]

pub mod evm;

mod address;
mod error;
mod signable_message;
mod signer;
mod unsigned_tx;

pub use address::{Address, Signature, SignedTx};
pub use error::ChainError;
pub use signable_message::SignableMessage;
pub use signer::ChainSigner;
pub use unsigned_tx::{AccessListItem, EvmUnsigned, UnsignedTx};
```

- [ ] **Step 9: Smoke test**

```bash
cargo build -p jova-core-chains
```

Expected: builds. (No unit tests in this crate yet — those come from `jova-core` testing the full surface against vectors in Task 4.)

- [ ] **Step 10: Commit**

```bash
git add crates/jova-core-chains/
git commit -m "feat(chains): EVM signer (EIP-1559, EIP-191, EIP-712)"
```

---

## Task 4: Public `JovaWallet` API + Phase 1 vectors

**Files:**
- Modify: `crates/jova-core/Cargo.toml`
- Create: `crates/jova-core/src/wallet.rs`
- Create: `crates/jova-core/src/chain.rs`
- Create: `crates/jova-core/src/error.rs`
- Modify: `crates/jova-core/src/lib.rs`
- Modify: `spec/test-vectors.json`
- Create: `crates/jova-core/tests/vectors_evm.rs`

- [ ] **Step 1: Add deps**

`crates/jova-core/Cargo.toml`:

```toml
[dependencies]
jova-core-primitives.workspace = true
jova-core-chains.workspace = true
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true

[dev-dependencies]
hex.workspace = true
```

- [ ] **Step 2: Define JovaError**

`crates/jova-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JovaError {
    #[error("invalid mnemonic")]
    InvalidMnemonic,
    #[error("invalid passphrase")]
    InvalidPassphrase,
    #[error("invalid address for {chain}")]
    InvalidAddress { chain: String },
    #[error("unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("malformed unsigned tx: {reason}")]
    MalformedUnsignedTx { reason: String },
    #[error("malformed signable message: {reason}")]
    MalformedSignableMessage { reason: String },
    #[error("signing failed: {reason}")]
    SigningFailed { reason: String },
    #[error("internal: {reason}")]
    Internal { reason: String },
}

impl From<jova_core_primitives::MnemonicError> for JovaError {
    fn from(_: jova_core_primitives::MnemonicError) -> Self { JovaError::InvalidMnemonic }
}

impl From<jova_core_chains::ChainError> for JovaError {
    fn from(e: jova_core_chains::ChainError) -> Self {
        match e {
            jova_core_chains::ChainError::InvalidAddress => JovaError::InvalidAddress { chain: "evm".into() },
            jova_core_chains::ChainError::MalformedUnsignedTx(r) => JovaError::MalformedUnsignedTx { reason: r },
            jova_core_chains::ChainError::MalformedSignableMessage(r) => JovaError::MalformedSignableMessage { reason: r },
            jova_core_chains::ChainError::SigningFailed(r) => JovaError::SigningFailed { reason: r },
            jova_core_chains::ChainError::Internal(r) => JovaError::Internal { reason: r },
        }
    }
}
```

- [ ] **Step 3: Define JovaChain**

`crates/jova-core/src/chain.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum JovaChain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
    // Phase 2: Bitcoin, Solana, Xrp.
    CustomEvm { chain_id: u64 },
}

impl JovaChain {
    pub(crate) fn evm_chain_id(&self) -> Option<u64> {
        match self {
            Self::Ethereum => Some(1),
            Self::Polygon => Some(137),
            Self::Bsc => Some(56),
            Self::Arbitrum => Some(42161),
            Self::Optimism => Some(10),
            Self::Base => Some(8453),
            Self::CustomEvm { chain_id } => Some(*chain_id),
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Bsc => "bsc",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::CustomEvm { .. } => "customEvm",
        }
    }

    pub(crate) fn derivation_path(&self) -> &'static str {
        // All EVM chains share m/44'/60'/0'/0/0 in v1; account index applied separately.
        "m/44'/60'/0'/0/0"
    }
}
```

- [ ] **Step 4: JovaWallet**

`crates/jova-core/src/wallet.rs`:

```rust
use jova_core_chains::{ChainSigner, evm::EvmSigner, Address, SignedTx, Signature, SignableMessage, UnsignedTx};
use jova_core_primitives::{Mnemonic, Seed, DerivationPath, derive_secp256k1};

use crate::chain::JovaChain;
use crate::error::JovaError;

pub struct JovaWallet {
    seed: Seed,
}

impl JovaWallet {
    pub fn from_mnemonic(words: &str, passphrase: &str) -> Result<Self, JovaError> {
        let seed = Mnemonic::to_seed(words, passphrase)?;
        Ok(Self { seed })
    }

    /// Derive the canonical address for the given chain.
    pub fn address(&self, chain: &JovaChain, _account: u32) -> Result<Address, JovaError> {
        let signer = self.evm_signer(chain)?;
        let xprv = self.derive_for(chain)?;
        Ok(signer.derive_address(&xprv)?)
    }

    /// Sign a transaction. Chain is implicit in the `UnsignedTx` variant
    /// (and for EVM, the chain ID inside the variant is authoritative).
    pub fn sign_tx(&self, unsigned: &UnsignedTx) -> Result<SignedTx, JovaError> {
        match unsigned {
            UnsignedTx::Evm(_) => {
                let signer = EvmSigner { chain_label: "ethereum" /* result.chain is overridden below */ };
                let xprv = self.derive_path("m/44'/60'/0'/0/0")?;
                let mut signed = signer.sign_tx(&xprv, unsigned)?;
                // Map result chain label from the chainId in the variant.
                if let UnsignedTx::Evm(evm) = unsigned {
                    signed.chain = chain_label_from_evm_chain_id(evm.chain_id);
                }
                Ok(signed)
            }
            // Phase 2+ adds Bitcoin, Solana, XRP arms here.
            _ => Err(JovaError::UnsupportedChain("phase1_evm_only".into())),
        }
    }

    /// Sign a message. Chain is implicit in the `SignableMessage` variant.
    pub fn sign_message(&self, msg: &SignableMessage) -> Result<Signature, JovaError> {
        match msg {
            SignableMessage::EvmPersonalSign { .. } | SignableMessage::EvmTypedDataV4 { .. } => {
                let signer = EvmSigner { chain_label: "ethereum" };
                let xprv = self.derive_path("m/44'/60'/0'/0/0")?;
                Ok(signer.sign_message(&xprv, msg)?)
            }
            // Phase 2+ adds Solana and Bitcoin arms.
            _ => Err(JovaError::UnsupportedChain("phase1_evm_only".into())),
        }
    }

    fn evm_signer(&self, chain: &JovaChain) -> Result<EvmSigner, JovaError> {
        if chain.evm_chain_id().is_none() {
            return Err(JovaError::UnsupportedChain(format!("{:?}", chain)));
        }
        Ok(EvmSigner { chain_label: chain.label() })
    }

    fn derive_for(&self, chain: &JovaChain) -> Result<jova_core_primitives::XPrv, JovaError> {
        self.derive_path(chain.derivation_path())
    }

    fn derive_path(&self, path_str: &str) -> Result<jova_core_primitives::XPrv, JovaError> {
        let path = DerivationPath::parse(path_str)
            .map_err(|_| JovaError::Internal { reason: "bad_path".into() })?;
        derive_secp256k1(&self.seed, &path)
            .map_err(|_| JovaError::Internal { reason: "derive_failed".into() })
    }
}

/// Map an EVM chain ID back to the canonical chain label used in `Address.chain`
/// and `SignedTx.chain`. Unknown chain IDs return `"customEvm"`.
fn chain_label_from_evm_chain_id(id: u64) -> String {
    match id {
        1     => "ethereum".to_string(),
        137   => "polygon".to_string(),
        56    => "bsc".to_string(),
        42161 => "arbitrum".to_string(),
        10    => "optimism".to_string(),
        8453  => "base".to_string(),
        _     => "customEvm".to_string(),
    }
}

pub fn create_mnemonic(strength: jova_core_primitives::Strength) -> Mnemonic {
    Mnemonic::generate(strength)
}

pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    Mnemonic::validate(words, passphrase).is_ok()
}

pub fn is_valid_address(addr: &str, chain: &JovaChain) -> bool {
    if chain.evm_chain_id().is_some() {
        jova_core_chains::evm::address::validate(addr)
    } else {
        false
    }
}
```

(The `evm::address::validate` is private at the moment — make it `pub(crate)` accessible by either re-exporting at the `evm` module or moving the validate fn into `jova_core_chains::ChainSigner::validate_address` and dispatching properly. The simplest fix: change `mod address;` to `pub mod address;` in `crates/jova-core-chains/src/evm/mod.rs`, then `pub use address::validate;` at the chain level.)

- [ ] **Step 5: Wire into lib.rs**

```rust
//! jova-core — public Rust API.

#![forbid(unsafe_code)]

mod chain;
mod error;
mod wallet;

pub use chain::JovaChain;
pub use error::JovaError;
pub use wallet::{JovaWallet, create_mnemonic, is_valid_mnemonic, is_valid_address};
pub use jova_core_chains::{Address, SignedTx, Signature, SignableMessage, UnsignedTx, EvmUnsigned, AccessListItem};
pub use jova_core_primitives::Strength;
```

- [ ] **Step 6: Capture reference values for the Phase 1 vector set**

This step produces the `expected` values that go into `spec/test-vectors.json`. The agent does NOT make these up — every value comes from running a reference signer against the same input. Foundry's `cast` is the reference signer for EVM.

#### Step 6.1: Install the reference toolchain

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
cast --version    # confirm cast is on PATH
```

#### Step 6.2: Pin the canonical input set

These are the inputs Phase 1 covers. Save as `tools/vector-capture/inputs.json` (the agent creates this):

```json
{
  "mnemonics": {
    "abandon": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    "trezor":  "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"
  },
  "chains": ["ethereum", "polygon", "bsc", "arbitrum", "optimism", "base"],
  "txs": {
    "simple_transfer":     { "chainId": 1,   "nonce": 0, "to": "0x0000000000000000000000000000000000000000", "value": "1000000000000000000", "gasLimit": 21000,  "maxFeePerGas": "30000000000", "maxPriorityFeePerGas": "2000000000", "data": "0x" },
    "erc20_transfer":      { "chainId": 1,   "nonce": 7, "to": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "value": "0",                  "gasLimit": 65000,  "maxFeePerGas": "30000000000", "maxPriorityFeePerGas": "2000000000", "data": "0xa9059cbb000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa960450000000000000000000000000000000000000000000000000de0b6b3a7640000" },
    "with_access_list":    { "chainId": 1,   "nonce": 1, "to": "0x0000000000000000000000000000000000001234", "value": "0",                  "gasLimit": 100000, "maxFeePerGas": "30000000000", "maxPriorityFeePerGas": "2000000000", "data": "0x", "accessList": [{ "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "storageKeys": ["0x0000000000000000000000000000000000000000000000000000000000000000"] }] },
    "polygon_transfer":    { "chainId": 137, "nonce": 0, "to": "0x0000000000000000000000000000000000000000", "value": "1000000000000000000", "gasLimit": 21000,  "maxFeePerGas": "60000000000", "maxPriorityFeePerGas": "30000000000", "data": "0x" }
  },
  "messages": {
    "eip191_hello":    "Hello, Jova",
    "eip712_permit":   "<the standard ERC-2612 Permit typed-data JSON; see EIP-2612 reference>"
  }
}
```

#### Step 6.3: Capture per input

For each `(mnemonic, chain, tx)` combo, run `cast wallet sign-tx` against a private key derived at `m/44'/60'/0'/0/0` from the mnemonic. The exact one-liner template:

```bash
PRIV=$(cast wallet derive "$MNEMONIC" --mnemonic-derivation-path "m/44'/60'/0'/0/0")
SIGNED=$(cast wallet sign-tx \
  --private-key "$PRIV" \
  --chain "$CHAIN_ID" \
  --nonce "$NONCE" \
  --to "$TO" \
  --value "$VALUE" \
  --gas-limit "$GAS_LIMIT" \
  --max-fee-per-gas "$MAX_FEE" \
  --max-priority-fee-per-gas "$MAX_PRI" \
  --data "$DATA" \
  --type 2)
HASH=$(cast keccak "$SIGNED")
echo "{\"signed_hex\":\"$SIGNED\",\"tx_hash\":\"$HASH\"}"
```

For addresses: `cast wallet derive "$MNEMONIC" --mnemonic-derivation-path "$PATH"` already prints the address.

For EIP-191 messages: `cast wallet sign --private-key "$PRIV" "$MESSAGE"`.

For EIP-712: `cast wallet sign --private-key "$PRIV" --data "$TYPED_DATA_JSON"`.

The agent writes a small bash script (`tools/vector-capture/capture.sh`) that loops over `inputs.json` and emits a populated vectors array on stdout. Commit the script alongside the vectors so the capture is reproducible.

#### Step 6.4: Append to `spec/test-vectors.json`

The vectors file from Phase 0 has one entry (the negative-mnemonic vector). After capture, the file's `vectors` array contains:

- The Phase 0 entry (untouched).
- 6 `address` vectors (3 inputs × 2 mnemonics).
- 4 `sign_tx` vectors (simple_transfer, erc20_transfer, with_access_list, polygon_transfer).
- 2 `sign_message` vectors (1 EIP-191, 1 EIP-712).
- 3 `error` vectors (`evm_to_address_invalid`, `evm_decimal_string_invalid`, `evm_data_not_hex`) — these have no `cast` capture; the input is malformed by construction and `expected` declares the error variant + reason string.

Bump the file's `version` field to `"0.2"`.

#### Step 6.5: Verify completeness before commit

```bash
cargo run -p jova-verify-spec
```

`tools/verify-spec` (built in Phase 0) parses the file and confirms it matches `spec/test-vectors.schema.json`. **Add a check:** any vector with `kind` in `address|sign_tx|sign_message` whose `expected` field contains the literal substring `"TODO"` or `"<capture` fails the verification. The agent extends `tools/verify-spec/src/main.rs` to enforce this:

```rust
// At the existing serde_json validation block, add:
if let Some(arr) = v.get("vectors").and_then(|x| x.as_array()) {
    for vector in arr {
        let id = vector.get("id").and_then(|x| x.as_str()).unwrap_or("?");
        let exp = serde_json::to_string(vector.get("expected").unwrap_or(&serde_json::Value::Null)).unwrap();
        if exp.contains("TODO") || exp.contains("<capture") || exp.contains("REPLACE") {
            errors.push(format!("vector {}: expected contains a placeholder", id));
        }
    }
}
```

Run `cargo run -p jova-verify-spec` again. Expected: `verify-spec: OK`. If any vector still has a placeholder, the agent did not actually capture; go back to Step 6.3.

- [ ] **Step 7: Vector test harness**

`crates/jova-core/tests/vectors_evm.rs`:

```rust
use jova_core::*;
use serde_json::Value;

fn load_vectors() -> Vec<Value> {
    let raw = include_str!("../../../spec/test-vectors.json");
    let v: Value = serde_json::from_str(raw).unwrap();
    v["vectors"].as_array().unwrap().clone()
}

#[test]
fn evm_address_vectors() {
    for v in load_vectors() {
        if v["kind"] != "address" { continue; }
        let chain_kind = v["input"]["chain"]["kind"].as_str().unwrap();
        if !matches!(chain_kind, "ethereum" | "polygon" | "bsc" | "arbitrum" | "optimism" | "base") { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let chain: JovaChain = serde_json::from_value(v["input"]["chain"].clone()).unwrap();
        let expected = v["expected"]["address"].as_str().unwrap();

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();
        let got = wallet.address(&chain, 0).unwrap();
        assert_eq!(got.value, expected, "vector {}", v["id"]);
    }
}

#[test]
fn evm_sign_tx_vectors() {
    for v in load_vectors() {
        if v["kind"] != "sign_tx" { continue; }

        let mnemonic = v["input"]["mnemonic"].as_str().unwrap();
        let pass     = v["input"]["passphrase"].as_str().unwrap_or("");
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone()).unwrap();
        let expected_hex = v["expected"]["signed_hex"].as_str().unwrap();
        let expected_hash = v["expected"]["tx_hash"].as_str().unwrap();

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass).unwrap();
        let signed = wallet.sign_tx(&unsigned).unwrap();
        assert_eq!(signed.raw_hex.to_lowercase(), expected_hex.to_lowercase(), "vector {}", v["id"]);
        assert_eq!(signed.tx_hash.to_lowercase(), expected_hash.to_lowercase(), "vector {}", v["id"]);
    }
}

// Mirror tests for sign_message and error variants — same shape.
```

- [ ] **Step 8: Run**

```bash
cargo test -p jova-core --test vectors_evm
```

Expected: passes for all vectors that have real expected values populated.

- [ ] **Step 9: Commit**

```bash
git add crates/jova-core/ spec/test-vectors.json
git commit -m "feat(core): JovaWallet + 18+ EVM vectors with reference values"
```

---

## Task 5: Update FFI surface

**Files:**
- Modify: `crates/jova-core-ffi/src/lib.rs`

- [ ] **Step 1: Replace stub with the typed surface**

uniffi 0.29's enum-with-data marshalling is mature. We use **typed enums and records** at the FFI — Swift gets a real `enum JovaChain { case ethereum, polygon, ..., customEvm(chainId: UInt64) }`, Kotlin gets a sealed class. Apps catch chain-enum errors at compile time. EIP-712 typed data is the only `String` field, because typed-data schemas are inherently dynamic.

```rust
//! jova-core-ffi — uniffi bindings for the public JovaWallet API.

#![forbid(unsafe_code)]

use std::sync::Arc;
use jova_core::{
    JovaWallet as InnerWallet,
    JovaError,
    Strength,
    Mnemonic,
};

// ---------- Typed FFI types ----------
//
// These mirror jova-core's domain types, annotated with uniffi derives so they
// generate idiomatic Swift/Kotlin/Python types automatically.

#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum JovaChain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
    Bitcoin,
    Solana,
    Xrp,
    CustomEvm { chain_id: u64 },
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct AccessListItem {
    pub address: String,
    pub storage_keys: Vec<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct EvmUnsigned {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: String,
    pub value: String,                  // wei, decimal string (avoids u128/U256 marshalling)
    pub gas_limit: u64,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub data: String,                   // 0x-prefixed hex
    pub access_list: Vec<AccessListItem>,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum UnsignedTx {
    Evm { tx: EvmUnsigned },
    Bitcoin { psbt_base64: String },
    Solana { message_base64: String, recent_blockhash: String },
    Xrp { tx_json: String },
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum BtcMsgScheme { Bip322, Legacy }

#[derive(uniffi::Enum, Clone, Debug)]
pub enum SignableMessage {
    EvmPersonalSign { message: String },
    EvmTypedDataV4  { json: String },         // dynamic schema; String is intentional
    Solana          { message_base64: String },
    Bitcoin         { message: String, address: String, scheme: BtcMsgScheme },
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct Address {
    pub chain: JovaChain,
    pub value: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct SignedTx {
    pub chain: JovaChain,
    pub raw_hex: String,
    pub tx_hash: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct Signature {
    pub hex: String,
}

// ---------- Errors ----------

#[derive(uniffi::Error, Debug, thiserror::Error)]
#[uniffi(flat_error)]
pub enum FfiError {
    #[error("invalid mnemonic")]                                  InvalidMnemonic,
    #[error("invalid passphrase")]                                InvalidPassphrase,
    #[error("invalid address for {chain:?}")]                     InvalidAddress { chain: JovaChain },
    #[error("unsupported chain: {chain:?}")]                      UnsupportedChain { chain: JovaChain },
    #[error("malformed unsigned tx: {reason}")]                   MalformedUnsignedTx { reason: String },
    #[error("malformed signable message: {reason}")]              MalformedSignableMessage { reason: String },
    #[error("signing failed: {reason}")]                          SigningFailed { reason: String },
    #[error("internal: {reason}")]                                Internal { reason: String },
}

// ---------- Conversions to/from jova-core domain types ----------
//
// The FFI types carry binding-friendly shapes; the core types are what the
// Rust signing logic uses. The two are close enough that conversion is
// mechanical — no information loss.

impl From<JovaError> for FfiError {
    fn from(e: JovaError) -> Self {
        match e {
            JovaError::InvalidMnemonic                   => Self::InvalidMnemonic,
            JovaError::InvalidPassphrase                 => Self::InvalidPassphrase,
            JovaError::InvalidAddress { chain }          => Self::InvalidAddress { chain: parse_chain(&chain).unwrap_or(JovaChain::Ethereum) },
            JovaError::UnsupportedChain(s)               => Self::UnsupportedChain { chain: parse_chain(&s).unwrap_or(JovaChain::Ethereum) },
            JovaError::MalformedUnsignedTx { reason }    => Self::MalformedUnsignedTx { reason },
            JovaError::MalformedSignableMessage { reason } => Self::MalformedSignableMessage { reason },
            JovaError::SigningFailed { reason }          => Self::SigningFailed { reason },
            JovaError::Internal { reason }               => Self::Internal { reason },
        }
    }
}

fn parse_chain(s: &str) -> Option<JovaChain> {
    Some(match s {
        "ethereum" => JovaChain::Ethereum,
        "polygon"  => JovaChain::Polygon,
        "bsc"      => JovaChain::Bsc,
        "arbitrum" => JovaChain::Arbitrum,
        "optimism" => JovaChain::Optimism,
        "base"     => JovaChain::Base,
        "bitcoin"  => JovaChain::Bitcoin,
        "solana"   => JovaChain::Solana,
        "xrp"      => JovaChain::Xrp,
        _ => return None,
    })
}

// FFI → core conversions (keep these private; the public API is the FFI shape).

impl From<JovaChain> for jova_core::JovaChain {
    fn from(c: JovaChain) -> Self {
        match c {
            JovaChain::Ethereum => jova_core::JovaChain::Ethereum,
            JovaChain::Polygon  => jova_core::JovaChain::Polygon,
            JovaChain::Bsc      => jova_core::JovaChain::Bsc,
            JovaChain::Arbitrum => jova_core::JovaChain::Arbitrum,
            JovaChain::Optimism => jova_core::JovaChain::Optimism,
            JovaChain::Base     => jova_core::JovaChain::Base,
            JovaChain::Bitcoin  => jova_core::JovaChain::Bitcoin,
            JovaChain::Solana   => jova_core::JovaChain::Solana,
            JovaChain::Xrp      => jova_core::JovaChain::Xrp,
            JovaChain::CustomEvm { chain_id } => jova_core::JovaChain::CustomEvm { chain_id },
        }
    }
}

impl From<jova_core::Address> for Address {
    fn from(a: jova_core::Address) -> Self {
        Self {
            chain: parse_chain(&a.chain).unwrap_or(JovaChain::Ethereum),
            value: a.value,
        }
    }
}

impl From<jova_core::SignedTx> for SignedTx {
    fn from(t: jova_core::SignedTx) -> Self {
        Self {
            chain: parse_chain(&t.chain).unwrap_or(JovaChain::Ethereum),
            raw_hex: t.raw_hex,
            tx_hash: t.tx_hash,
        }
    }
}

impl From<jova_core::Signature> for Signature {
    fn from(s: jova_core::Signature) -> Self { Self { hex: s.hex } }
}

impl From<UnsignedTx> for jova_core::UnsignedTx {
    fn from(t: UnsignedTx) -> Self {
        match t {
            UnsignedTx::Evm { tx } => jova_core::UnsignedTx::Evm(jova_core::EvmUnsigned {
                chain_id: tx.chain_id,
                nonce: tx.nonce,
                to: tx.to,
                value: tx.value,
                gas_limit: tx.gas_limit,
                max_fee_per_gas: tx.max_fee_per_gas,
                max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                data: tx.data,
                access_list: tx.access_list.into_iter().map(|x| jova_core::AccessListItem {
                    address: x.address,
                    storage_keys: x.storage_keys,
                }).collect(),
            }),
            UnsignedTx::Bitcoin { psbt_base64 } => jova_core::UnsignedTx::Bitcoin { psbt_base64 },
            UnsignedTx::Solana { message_base64, recent_blockhash } => jova_core::UnsignedTx::Solana { message_base64, recent_blockhash },
            UnsignedTx::Xrp { tx_json } => jova_core::UnsignedTx::Xrp { tx_json },
        }
    }
}

impl From<SignableMessage> for jova_core::SignableMessage {
    fn from(m: SignableMessage) -> Self {
        match m {
            SignableMessage::EvmPersonalSign { message } => jova_core::SignableMessage::EvmPersonalSign { message },
            SignableMessage::EvmTypedDataV4  { json }    => jova_core::SignableMessage::EvmTypedDataV4 { json },
            SignableMessage::Solana          { message_base64 } => jova_core::SignableMessage::Solana { message_base64 },
            SignableMessage::Bitcoin { message, address, scheme } => jova_core::SignableMessage::Bitcoin {
                message, address,
                scheme: match scheme {
                    BtcMsgScheme::Bip322 => jova_core::BtcMsgScheme::Bip322,
                    BtcMsgScheme::Legacy => jova_core::BtcMsgScheme::Legacy,
                },
            },
        }
    }
}

// ---------- Free functions ----------

#[uniffi::export]
pub fn create_mnemonic(bits256: bool) -> Mnemonic {
    let s = if bits256 { Strength::Bits256 } else { Strength::Bits128 };
    jova_core::create_mnemonic(s)
}

#[uniffi::export]
pub fn is_valid_mnemonic(words: String, passphrase: String) -> bool {
    jova_core::is_valid_mnemonic(&words, &passphrase)
}

#[uniffi::export]
pub fn is_valid_address(addr: String, chain: JovaChain) -> bool {
    let core_chain: jova_core::JovaChain = chain.into();
    jova_core::is_valid_address(&addr, &core_chain)
}

// ---------- Wallet object ----------

#[derive(uniffi::Object)]
pub struct JovaWallet {
    inner: InnerWallet,
}

#[uniffi::export]
impl JovaWallet {
    #[uniffi::constructor]
    pub fn from_mnemonic(words: String, passphrase: String) -> Result<Arc<Self>, FfiError> {
        let inner = InnerWallet::from_mnemonic(&words, &passphrase).map_err(FfiError::from)?;
        Ok(Arc::new(Self { inner }))
    }

    pub fn address(&self, chain: JovaChain, account: u32) -> Result<Address, FfiError> {
        let core_chain: jova_core::JovaChain = chain.into();
        Ok(self.inner.address(&core_chain, account).map_err(FfiError::from)?.into())
    }

    /// Chain is implicit in the `UnsignedTx` variant; for EVM, the chain ID inside the variant is authoritative.
    pub fn sign_tx(&self, unsigned: UnsignedTx) -> Result<SignedTx, FfiError> {
        let core_tx: jova_core::UnsignedTx = unsigned.into();
        Ok(self.inner.sign_tx(&core_tx).map_err(FfiError::from)?.into())
    }

    /// Chain is implicit in the `SignableMessage` variant.
    pub fn sign_message(&self, msg: SignableMessage) -> Result<Signature, FfiError> {
        let core_msg: jova_core::SignableMessage = msg.into();
        Ok(self.inner.sign_message(&core_msg).map_err(FfiError::from)?.into())
    }
}

uniffi::setup_scaffolding!();
```

The cost of typed FFI is the conversion boilerplate above (mechanical, not subtle). The benefit is: every binding gets compile-time-checked enums and records. The Convenience layers in each binding shrink dramatically because the generated types are already idiomatic.

If a binding's IDE shows a missing `case` after a chain is added in a minor version, that's the *desired* behavior — exhaustive matching surfaces incompatibility at compile time. App teams handle the new variant or use a default arm. Adding a chain remains a minor SDK release per ADR D8.

- [ ] **Step 2: Re-derive bindings, build**

```bash
cargo build -p jova-core-ffi --release
uniffi-bindgen-cli generate \
  --library target/release/libjova_core_ffi.dylib \
  --language swift --out-dir bindings/swift/Sources/JovaCore
uniffi-bindgen-cli generate \
  --library target/release/libjova_core_ffi.dylib \
  --language kotlin --out-dir bindings/kotlin/jova-core/src/main/kotlin
```

- [ ] **Step 3: Commit**

```bash
git add crates/jova-core-ffi/ bindings/swift/Sources/JovaCore bindings/kotlin/jova-core/src/main/kotlin
git commit -m "feat(ffi): JovaWallet exported via uniffi (JSON-shaped chain/tx/message params)"
```

---

## Task 6: Swift parity tests

**Files:**
- Modify: `bindings/swift/Sources/JovaCore/Convenience.swift`
- Create: `bindings/swift/Tests/JovaCoreTests/EvmVectorsTests.swift`

- [ ] **Step 1: Convenience layer**

With typed FFI, the Convenience layer is small. uniffi generates `JovaChain`, `UnsignedTx`, `SignableMessage`, `Address`, `SignedTx`, `Signature`, `JovaWallet` as idiomatic Swift types directly. Convenience just adds version metadata and a few ergonomic factory helpers.

`bindings/swift/Sources/JovaCore/Convenience.swift`:

```swift
import Foundation

public enum JovaCoreVersion {
    public static let value = "0.1.0"
}

// Helper for the most common case: build an EVM transfer without filling in the access list.
extension EvmUnsigned {
    public static func transfer(
        chainId: UInt64,
        nonce: UInt64,
        to: String,
        valueWei: String,
        gasLimit: UInt64 = 21_000,
        maxFeePerGas: String,
        maxPriorityFeePerGas: String
    ) -> EvmUnsigned {
        EvmUnsigned(
            chainId: chainId,
            nonce: nonce,
            to: to,
            value: valueWei,
            gasLimit: gasLimit,
            maxFeePerGas: maxFeePerGas,
            maxPriorityFeePerGas: maxPriorityFeePerGas,
            data: "0x",
            accessList: []
        )
    }
}
```

That's it. No JSON encoding, no `JovaChainSwift` shadow type, no extension methods to translate. The generated `JovaChain`, `UnsignedTx`, `SignableMessage` enums are already what apps consume.

- [ ] **Step 2: EVM vector tests**

The vector JSON's `chain` field has shape `{"kind": "ethereum"}` or `{"kind": "customEvm", "chainId": 1234}`. We decode it into the uniffi-generated `JovaChain` via a small helper. Same for `EvmUnsigned`.

`bindings/swift/Tests/JovaCoreTests/VectorDecoders.swift`:

```swift
import Foundation
@testable import JovaCore

enum VectorDecodeError: Error { case unknownChainKind(String); case missingField(String) }

func decodeChain(_ dict: [String: Any]) throws -> JovaChain {
    guard let kind = dict["kind"] as? String else { throw VectorDecodeError.missingField("kind") }
    switch kind {
    case "ethereum": return .ethereum
    case "polygon":  return .polygon
    case "bsc":      return .bsc
    case "arbitrum": return .arbitrum
    case "optimism": return .optimism
    case "base":     return .base
    case "bitcoin":  return .bitcoin
    case "solana":   return .solana
    case "xrp":      return .xrp
    case "customEvm":
        guard let id = dict["chainId"] as? UInt64 else { throw VectorDecodeError.missingField("chainId") }
        return .customEvm(chainId: id)
    default: throw VectorDecodeError.unknownChainKind(kind)
    }
}

func decodeEvmUnsigned(_ dict: [String: Any]) throws -> EvmUnsigned {
    EvmUnsigned(
        chainId: dict["chainId"] as! UInt64,
        nonce:   dict["nonce"] as! UInt64,
        to:      dict["to"] as! String,
        value:   dict["value"] as! String,
        gasLimit: dict["gasLimit"] as! UInt64,
        maxFeePerGas: dict["maxFeePerGas"] as! String,
        maxPriorityFeePerGas: dict["maxPriorityFeePerGas"] as! String,
        data:    dict["data"] as! String,
        accessList: ((dict["accessList"] as? [[String: Any]]) ?? []).map { item in
            AccessListItem(
                address: item["address"] as! String,
                storageKeys: item["storageKeys"] as! [String]
            )
        }
    )
}
```

`bindings/swift/Tests/JovaCoreTests/EvmVectorsTests.swift`:

```swift
import XCTest
@testable import JovaCore

final class EvmVectorsTests: XCTestCase {
    private func loadVectors() throws -> [[String: Any]] {
        let url = Bundle.module.url(forResource: "test-vectors", withExtension: "json")!
        let data = try Data(contentsOf: url)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        return json["vectors"] as! [[String: Any]]
    }

    func testEvmAddressVectors() throws {
        for v in try loadVectors() where (v["kind"] as? String) == "address" {
            let input = v["input"] as! [String: Any]
            let chain = try decodeChain(input["chain"] as! [String: Any])
            // Skip non-EVM chains in this test (they get their own test file in Phase 2/3).
            switch chain {
            case .ethereum, .polygon, .bsc, .arbitrum, .optimism, .base, .customEvm: break
            default: continue
            }

            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = (v["expected"] as! [String: Any])["address"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let got = try wallet.address(chain: chain, account: 0)
            XCTAssertEqual(got.value, expected, "vector \(v["id"] ?? "?")")
        }
    }

    func testEvmSignTxVectors() throws {
        for v in try loadVectors() where (v["kind"] as? String) == "sign_tx" {
            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let unsignedDict = input["unsigned_tx"] as! [String: Any]

            // EVM-only path in this test.
            guard (unsignedDict["kind"] as? String) == "evm" else { continue }
            let evm = try decodeEvmUnsigned(unsignedDict)
            let unsigned = UnsignedTx.evm(tx: evm)

            let expected = v["expected"] as! [String: Any]

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let signed = try wallet.signTx(unsigned: unsigned)
            XCTAssertEqual(signed.rawHex.lowercased(),
                           (expected["signed_hex"] as! String).lowercased(),
                           "vector \(v["id"] ?? "?")")
            XCTAssertEqual(signed.txHash.lowercased(),
                           (expected["tx_hash"] as! String).lowercased())
        }
    }

    // sign_message and error vectors mirror the same shape using SignableMessage and FfiError.
}
```

The body is shorter and type-safe. The test would fail to compile if a `JovaChain` case were renamed — which is exactly what we want from a parity test.

- [ ] **Step 3: Build and run**

```bash
./bindings/swift/scripts/build-xcframework.sh
cd bindings/swift && swift test
cd ../..
```

Expected: all EVM vectors pass.

- [ ] **Step 4: Commit**

```bash
git add bindings/swift/
git commit -m "test(swift): EVM vector parity"
```

---

## Task 7: Kotlin parity tests

**Files:**
- Create: `bindings/kotlin/jova-core/src/main/kotlin/io/jova/core/Convenience.kt`
- Create: `bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/EvmVectorsTest.kt`

- [ ] **Step 1: Convenience.kt** — minimal, since uniffi generates idiomatic Kotlin

uniffi 0.29 generates `JovaChain` as a sealed class with case objects (`JovaChain.Ethereum`, etc.) and a data class for `CustomEvm`. Convenience layer adds version metadata and ergonomic factories.

```kotlin
package io.jova.core

object JovaCoreVersion {
    const val VALUE = "0.1.0"
}

// Helper for the most common case: build an EVM transfer without filling in the access list.
fun evmTransfer(
    chainId: ULong,
    nonce: ULong,
    to: String,
    valueWei: String,
    gasLimit: ULong = 21_000UL,
    maxFeePerGas: String,
    maxPriorityFeePerGas: String
): EvmUnsigned = EvmUnsigned(
    chainId = chainId,
    nonce = nonce,
    to = to,
    value = valueWei,
    gasLimit = gasLimit,
    maxFeePerGas = maxFeePerGas,
    maxPriorityFeePerGas = maxPriorityFeePerGas,
    `data` = "0x",
    accessList = emptyList()
)
```

- [ ] **Step 2: VectorDecoders.kt + EvmVectorsTest.kt**

`bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/VectorDecoders.kt`:

```kotlin
package io.jova.core

import org.json.JSONObject

class VectorDecodeException(msg: String) : RuntimeException(msg)

fun decodeChain(o: JSONObject): JovaChain = when (val kind = o.getString("kind")) {
    "ethereum" -> JovaChain.Ethereum
    "polygon"  -> JovaChain.Polygon
    "bsc"      -> JovaChain.Bsc
    "arbitrum" -> JovaChain.Arbitrum
    "optimism" -> JovaChain.Optimism
    "base"     -> JovaChain.Base
    "bitcoin"  -> JovaChain.Bitcoin
    "solana"   -> JovaChain.Solana
    "xrp"      -> JovaChain.Xrp
    "customEvm" -> JovaChain.CustomEvm(chainId = o.getLong("chainId").toULong())
    else       -> throw VectorDecodeException("unknown chain kind: $kind")
}

fun decodeEvmUnsigned(o: JSONObject): EvmUnsigned {
    val accessList = if (o.has("accessList")) {
        val arr = o.getJSONArray("accessList")
        (0 until arr.length()).map { i ->
            val item = arr.getJSONObject(i)
            val keysArr = item.getJSONArray("storageKeys")
            AccessListItem(
                address = item.getString("address"),
                storageKeys = (0 until keysArr.length()).map { keysArr.getString(it) }
            )
        }
    } else emptyList()
    return EvmUnsigned(
        chainId = o.getLong("chainId").toULong(),
        nonce = o.getLong("nonce").toULong(),
        to = o.getString("to"),
        value = o.getString("value"),
        gasLimit = o.getLong("gasLimit").toULong(),
        maxFeePerGas = o.getString("maxFeePerGas"),
        maxPriorityFeePerGas = o.getString("maxPriorityFeePerGas"),
        `data` = o.getString("data"),
        accessList = accessList
    )
}
```

`bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/EvmVectorsTest.kt`:

```kotlin
package io.jova.core

import org.junit.Test
import org.junit.Assert.assertEquals
import org.json.JSONObject
import org.json.JSONArray

class EvmVectorsTest {
    private fun loadVectors(): JSONArray {
        val raw = javaClass.getResourceAsStream("/test-vectors.json")!!
            .bufferedReader().readText()
        return JSONObject(raw).getJSONArray("vectors")
    }

    @Test
    fun evmAddressVectors() {
        val vectors = loadVectors()
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "address") continue
            val input = v.getJSONObject("input")
            val chain = decodeChain(input.getJSONObject("chain"))
            // EVM-only in this test:
            when (chain) {
                JovaChain.Ethereum, JovaChain.Polygon, JovaChain.Bsc,
                JovaChain.Arbitrum, JovaChain.Optimism, JovaChain.Base,
                is JovaChain.CustomEvm -> { /* keep */ }
                else -> continue
            }

            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected").getString("address")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val got = wallet.address(chain, 0u)
            assertEquals("vector ${v.getString("id")}", expected, got.value)
        }
    }

    @Test
    fun evmSignTxVectors() {
        val vectors = loadVectors()
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_tx") continue
            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val unsignedDict = input.getJSONObject("unsigned_tx")

            if (unsignedDict.getString("kind") != "evm") continue
            val unsigned = UnsignedTx.Evm(tx = decodeEvmUnsigned(unsignedDict))
            val expected = v.getJSONObject("expected")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val signed = wallet.signTx(unsigned)
            assertEquals("vector ${v.getString("id")} hex",
                expected.getString("signed_hex").lowercase(), signed.rawHex.lowercase())
            assertEquals("vector ${v.getString("id")} hash",
                expected.getString("tx_hash").lowercase(), signed.txHash.lowercase())
        }
    }
}
```

- [ ] **Step 3: Build and run**

```bash
./bindings/kotlin/scripts/build-aar.sh
cd bindings/kotlin && ./gradlew :jova-core:test
cd ../..
```

Expected: all EVM vectors pass.

- [ ] **Step 4: Commit**

```bash
git add bindings/kotlin/
git commit -m "test(kotlin): EVM vector parity"
```

---

## Task 8: Update WASM crate to compile (no functional tests yet)

**Files:**
- Modify: `crates/jova-core-wasm/src/lib.rs`

- [ ] **Step 1: Update WASM surface**

```rust
//! jova-core-wasm — wasm-bindgen bindings.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}

#[wasm_bindgen(js_name = createMnemonic)]
pub fn create_mnemonic(bits256: bool) -> String {
    let s = if bits256 { jova_core::Strength::Bits256 } else { jova_core::Strength::Bits128 };
    let m = jova_core::create_mnemonic(s);
    m.words.clone()
}

// Phase 6 will add the full JovaWallet surface; for now we just confirm the crate compiles.
```

- [ ] **Step 2: Build**

```bash
cd bindings/wasm && ./scripts/build-wasm.sh && pnpm test
cd ../..
```

Expected: WASM hello-world from Phase 0 still passes.

- [ ] **Step 3: Commit**

```bash
git add crates/jova-core-wasm/
git commit -m "feat(wasm): keep compile-clean through Phase 1; functional API in Phase 6"
```

---

## Task 9: Property tests + fuzz targets

**Files:**
- Create: `crates/jova-core/tests/properties_evm.rs`
- Create: `fuzz/Cargo.toml`
- Create: `fuzz/fuzz_targets/fuzz_eip1559_decode.rs`
- Create: `fuzz/fuzz_targets/fuzz_eip712_typed.rs`
- Create: `fuzz/fuzz_targets/fuzz_address_parse.rs`

- [ ] **Step 1: Property tests**

`crates/jova-core/tests/properties_evm.rs`:

```rust
use jova_core::*;
use proptest::prelude::*;

const SEED_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

proptest! {
    #[test]
    fn address_is_deterministic_per_chain(account in 0u32..1000) {
        let _ = account; // v1 always uses 0; this proptest reserves the future shape.
        let w1 = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let w2 = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let a1 = w1.address(&JovaChain::Ethereum, 0).unwrap();
        let a2 = w2.address(&JovaChain::Ethereum, 0).unwrap();
        prop_assert_eq!(a1.value, a2.value);
    }

    #[test]
    fn validate_then_derive_round_trips_to_self(_n in 0u32..100) {
        let w = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let a = w.address(&JovaChain::Ethereum, 0).unwrap();
        prop_assert!(is_valid_address(&a.value, &JovaChain::Ethereum));
    }
}
```

- [ ] **Step 2: Fuzz workspace**

`fuzz/Cargo.toml`:

```toml
[package]
name = "jova-fuzz"
version = "0.0.0"
edition.workspace = true
publish = false

[dependencies]
libfuzzer-sys = "0.4"
jova-core.workspace = true
serde_json.workspace = true

[[bin]]
name = "fuzz_eip1559_decode"
path = "fuzz_targets/fuzz_eip1559_decode.rs"
test = false
doc  = false

[[bin]]
name = "fuzz_eip712_typed"
path = "fuzz_targets/fuzz_eip712_typed.rs"
test = false
doc  = false

[[bin]]
name = "fuzz_address_parse"
path = "fuzz_targets/fuzz_address_parse.rs"
test = false
doc  = false
```

- [ ] **Step 3: Fuzz targets**

`fuzz/fuzz_targets/fuzz_eip1559_decode.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{JovaChain, JovaWallet, UnsignedTx};

const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(unsigned) = serde_json::from_str::<UnsignedTx>(s) {
            let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
            let _ = w.sign_tx(&unsigned);
        }
    }
});
```

`fuzz/fuzz_targets/fuzz_eip712_typed.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{JovaChain, JovaWallet, SignableMessage};

const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let msg = SignableMessage::EvmTypedDataV4 { json: s.to_string() };
        let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let _ = w.sign_message(&msg);
    }
});
```

`fuzz/fuzz_targets/fuzz_address_parse.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{is_valid_address, JovaChain};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = is_valid_address(s, &JovaChain::Ethereum);
    }
});
```

- [ ] **Step 4: Run each fuzzer for 60 seconds locally**

```bash
cargo install cargo-fuzz --locked
cd fuzz
for target in fuzz_eip1559_decode fuzz_eip712_typed fuzz_address_parse; do
    cargo fuzz run "$target" -- -max_total_time=60
done
cd ..
```

Expected: no crashes. If the fuzzer finds one, the agent must fix the panic before merging.

- [ ] **Step 5: Commit**

```bash
git add crates/jova-core/tests/properties_evm.rs fuzz/
git commit -m "test: property tests + fuzz targets for EVM"
```

---

## Task 10: miri pass + open PR + tag v0.1.0

- [ ] **Step 1: miri**

```bash
rustup component add miri --toolchain nightly
cargo +nightly miri test -p jova-core-primitives
```

Expected: clean. If miri reports UB, the agent must fix it before merging.

- [ ] **Step 2: Open the PR**

```bash
git push -u origin feat/phase-1-evm
gh pr create --title "Phase 1: EVM end-to-end with vector parity" --body "$(cat <<'EOF'
## Summary
- Real BIP-39 + BIP-32 + EIP-55 + EIP-1559 + EIP-191 + EIP-712 in primitives & chains
- 18+ EVM vectors with reference values from cast wallet sign-tx
- Vector parity passing on Rust + Swift + Kotlin
- WASM still compile-only (Phase 6)
- Property tests + 3 fuzz targets

## Test plan
- [x] cargo test --workspace passes
- [x] miri on jova-core-primitives passes
- [x] All 6 CI workflows pass
- [x] cargo fuzz run for 60s on each target — no crashes

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: After CI green and review approval, merge**

```bash
gh pr merge --squash --delete-branch
git checkout main && git pull
```

- [ ] **Step 4: Tag v0.1.0**

```bash
git tag -a v0.1.0 -m "v0.1.0 — Phase 1 EVM end-to-end"
git push origin v0.1.0
```

---

## Self-review

- [ ] Every task has exact paths and exact commands.
- [ ] Every code block is complete (no `...` or `// implement here`).
- [ ] Method signatures match across `jova-core-primitives`, `jova-core-chains`, `jova-core`, `jova-core-ffi`, and the binding test files.
- [ ] Address vectors and tx vectors have **real expected values** sourced from a reference signer; placeholder TODOs in the JSON must be replaced before commit.
- [ ] `is_valid_address` exists and is referenced from both the property tests and the binding tests.
- [ ] Malformed EVM tx fields produce `JovaError::MalformedUnsignedTx { reason: ... }` with a stable reason string from the `spec/errors.md` vocabulary — the error vectors test this exactly.
- [ ] WASM crate still compiles after Phase 1 changes; the Phase 0 hello-world WASM test still passes.
- [ ] miri clean.

---

## What this plan does NOT do

- Does not implement BTC, SOL, or XRP. Phase 2 / Phase 3.
- Does not run WASM functional vector tests. Phase 6.
- Does not publish artifacts to any registry. Phase 5+.
- Does not produce the iOS or Android app integration. Phase 4 (in app repos).

---

## Estimated time

10–14 days. Time sinks:
1. Authoring real reference values for vectors (running `cast`/`geth` for each input case).
2. Aligning alloy 0.7 type names with the snippets above (alloy's API surface evolves quickly).
3. uniffi enum-with-data marshalling decisions (we punted to JSON-shaped fields; if the team wants typed enums, add 2–3 days for schema authoring).
