//! Property tests for the Bitcoin signing surface.
//!
//! These are not byte-equal vector checks (those live in `vectors_btc.rs`).
//! They're invariant-style assertions: determinism, validator robustness,
//! no panics on adversarial inputs.

use jova_core::{JovaChain, JovaWallet, is_valid_address};
use proptest::prelude::*;

const SEED_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const SEED_B: &str = "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic";

proptest! {
    // Pure-Rust determinism check: same mnemonic → same BTC address every time.
    #[test]
    fn btc_address_is_deterministic(_iter in 0u32..32) {
        let a = JovaWallet::from_mnemonic(SEED_A, "").unwrap()
            .address(&JovaChain::Bitcoin, 0).unwrap();
        let b = JovaWallet::from_mnemonic(SEED_A, "").unwrap()
            .address(&JovaChain::Bitcoin, 0).unwrap();
        prop_assert_eq!(a.value, b.value);
    }

    // Different mnemonics must produce different addresses.
    #[test]
    fn btc_distinct_mnemonics_produce_distinct_addresses(_iter in 0u32..32) {
        let a = JovaWallet::from_mnemonic(SEED_A, "").unwrap()
            .address(&JovaChain::Bitcoin, 0).unwrap();
        let b = JovaWallet::from_mnemonic(SEED_B, "").unwrap()
            .address(&JovaChain::Bitcoin, 0).unwrap();
        prop_assert_ne!(a.value, b.value);
    }

    // The derived address always validates.
    #[test]
    fn btc_derived_address_self_validates(_iter in 0u32..32) {
        let a = JovaWallet::from_mnemonic(SEED_A, "").unwrap()
            .address(&JovaChain::Bitcoin, 0).unwrap();
        prop_assert!(is_valid_address(&a.value, &JovaChain::Bitcoin));
    }

    // is_valid_address never panics on arbitrary input. Almost all random
    // strings will return false (a vanishingly small fraction by chance might
    // be valid bech32 P2WPKH — we don't assert false, only no-panic).
    #[test]
    fn btc_validator_never_panics_on_random_strings(s in "\\PC*") {
        let _ = is_valid_address(&s, &JovaChain::Bitcoin);
    }

    // is_valid_address never panics on arbitrary bytes that happen to be UTF-8.
    #[test]
    fn btc_validator_never_panics_on_random_utf8(s in "[\\x20-\\x7e]{0,128}") {
        let _ = is_valid_address(&s, &JovaChain::Bitcoin);
    }

    // EVM addresses must never validate as BTC and vice versa.
    #[test]
    fn btc_validator_rejects_evm_shape(s in "0x[0-9a-fA-F]{40}") {
        prop_assert!(!is_valid_address(&s, &JovaChain::Bitcoin));
    }
}
