//! Phase 7: hardware-wallet seed-bytes constructor.
//!
//! `JovaWallet::from_seed_bytes` accepts a pre-derived BIP-39 seed (64 bytes)
//! and bypasses the mnemonic → PBKDF2 step. Firmware that has already derived
//! the seed in a secure element constructs the wallet this way. Address
//! derivation must match the from_mnemonic path byte-for-byte.

#![cfg(feature = "external-rng")]

use jova_core::{JovaChain, JovaWallet};

const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The PBKDF2-derived seed for the abandon-about mnemonic with empty passphrase.
/// 64 bytes, hex-encoded. Captured from the existing `from_mnemonic` path.
const ABANDON_SEED_HEX: &str = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc1\
     9a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";

fn hex_to_seed(hex: &str) -> [u8; 64] {
    let bytes = hex::decode(hex.trim()).expect("hex decode");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&bytes);
    seed
}

#[test]
fn from_seed_bytes_matches_from_mnemonic_address() {
    let from_mnemonic = JovaWallet::from_mnemonic(ABANDON_MNEMONIC, "")
        .expect("mnemonic valid")
        .address(&JovaChain::Ethereum, 0)
        .expect("derive");

    let seed_bytes = hex_to_seed(ABANDON_SEED_HEX);
    let from_seed = JovaWallet::from_seed_bytes(seed_bytes)
        .address(&JovaChain::Ethereum, 0)
        .expect("derive");

    assert_eq!(
        from_mnemonic.value, from_seed.value,
        "from_seed_bytes and from_mnemonic must derive the same address"
    );
    assert_eq!(
        from_mnemonic.value, "0x9858EfFD232B4033E47d90003D41EC34EcaEda94",
        "address matches the Phase 1 EVM vector"
    );
}

#[test]
fn from_seed_bytes_matches_for_bitcoin() {
    let from_mnemonic = JovaWallet::from_mnemonic(ABANDON_MNEMONIC, "")
        .expect("mnemonic valid")
        .address(&JovaChain::Bitcoin, 0)
        .expect("derive");

    let seed_bytes = hex_to_seed(ABANDON_SEED_HEX);
    let from_seed = JovaWallet::from_seed_bytes(seed_bytes)
        .address(&JovaChain::Bitcoin, 0)
        .expect("derive");

    assert_eq!(from_mnemonic.value, from_seed.value);
    assert_eq!(
        from_mnemonic.value,
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
    );
}
