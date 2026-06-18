//! Track 0: private-key import unit tests (secp256k1 chains in this file;
//! Solana ed25519 covered in the same file once Task 3 lands).
//!
//! Known vector: the secp256k1 test scalar
//! 0x4646464646464646464646464646464646464646464646464646464646464646
//! (EIP-155 example key) maps to EVM address
//! 0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F.

use jova_core::{JovaChain, JovaError, JovaWallet};

/// EIP-155 example private key (32 bytes of 0x46).
const EIP155_KEY: &str =
    "4646464646464646464646464646464646464646464646464646464646464646";
const EIP155_EVM_ADDR: &str = "0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F";

#[test]
fn from_private_key_evm_derives_known_address() {
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid secp256k1 key");
    let addr = wallet.address(&JovaChain::Ethereum, 0).expect("derive");
    assert_eq!(
        addr.value.to_lowercase(),
        EIP155_EVM_ADDR.to_lowercase(),
        "EVM address from imported key must match the EIP-155 example"
    );
}

#[test]
fn from_private_key_accepts_0x_prefix() {
    let with_prefix = format!("0x{EIP155_KEY}");
    let wallet = JovaWallet::from_private_key(&with_prefix, &JovaChain::Ethereum)
        .expect("0x prefix tolerated");
    let addr = wallet.address(&JovaChain::Ethereum, 0).expect("derive");
    assert_eq!(addr.value.to_lowercase(), EIP155_EVM_ADDR.to_lowercase());
}

#[test]
fn from_private_key_rejects_bad_hex() {
    let err = JovaWallet::from_private_key("zz46", &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { .. }),
        "non-hex must be InvalidPrivateKey, got {err:?}"
    );
}

#[test]
fn from_private_key_rejects_wrong_length() {
    // 31 bytes.
    let short = "46".repeat(31);
    let err = JovaWallet::from_private_key(&short, &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { .. }),
        "31-byte key must be InvalidPrivateKey, got {err:?}"
    );
}

#[test]
fn from_private_key_rejects_zero_scalar() {
    let zero = "00".repeat(32);
    let err = JovaWallet::from_private_key(&zero, &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { .. }),
        "zero scalar is not a valid secp256k1 key, got {err:?}"
    );
}

#[test]
fn key_material_wallet_rejects_unbound_chain() {
    // An Ethereum-bound key must refuse to derive a Polygon address even
    // though both are secp256k1 — strict single-chain scoping.
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid key");
    let err = wallet.address(&JovaChain::Polygon, 0).unwrap_err();
    assert!(
        matches!(err, JovaError::UnsupportedChain(_)),
        "bound=ethereum, asked=polygon must be UnsupportedChain, got {err:?}"
    );
}
