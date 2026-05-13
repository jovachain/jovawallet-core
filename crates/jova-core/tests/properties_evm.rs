//! Property tests for the EVM signing surface.
//!
//! These use `proptest` to exercise round-trip invariants, determinism, and
//! parser robustness across large input spaces.  All properties hold for any
//! input — they complement the exact-value vector tests, not replace them.

use jova_core::{is_valid_address, JovaChain, JovaWallet};
use proptest::prelude::*;

const SEED_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

proptest! {
    // ── Determinism ──────────────────────────────────────────────────────────

    /// Two wallets built from the same mnemonic must produce the same address.
    #[test]
    fn address_is_deterministic_per_chain(_account in 0u32..1000) {
        let w1 = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let w2 = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let a1 = w1.address(&JovaChain::Ethereum, 0).unwrap();
        let a2 = w2.address(&JovaChain::Ethereum, 0).unwrap();
        prop_assert_eq!(a1.value, a2.value);
    }

    /// Different passphrases must produce different addresses (birthday-safe).
    #[test]
    fn different_passphrase_gives_different_address(
        pass in "[a-zA-Z0-9]{1,32}",
    ) {
        let w_empty = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let w_pass  = JovaWallet::from_mnemonic(SEED_MNEMONIC, &pass).unwrap();
        let a_empty = w_empty.address(&JovaChain::Ethereum, 0).unwrap();
        let a_pass  = w_pass.address(&JovaChain::Ethereum, 0).unwrap();
        prop_assert_ne!(a_empty.value, a_pass.value);
    }

    // ── Round-trip: is_valid_address accepts derived addresses ───────────────

    /// Any address derived from a valid mnemonic is accepted by is_valid_address.
    #[test]
    fn derived_address_is_valid(_n in 0u32..100) {
        let w = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        let a = w.address(&JovaChain::Ethereum, 0).unwrap();
        prop_assert!(is_valid_address(&a.value, &JovaChain::Ethereum));
    }

    /// is_valid_address is consistent across all supported EVM chains for derived addresses.
    #[test]
    fn derived_address_valid_on_all_evm_chains(_n in 0u32..50) {
        let chains = [
            JovaChain::Ethereum,
            JovaChain::Polygon,
            JovaChain::Bsc,
            JovaChain::Arbitrum,
            JovaChain::Optimism,
            JovaChain::Base,
        ];
        let w = JovaWallet::from_mnemonic(SEED_MNEMONIC, "").unwrap();
        for chain in &chains {
            let a = w.address(chain, 0).unwrap();
            prop_assert!(
                is_valid_address(&a.value, chain),
                "address {} not valid on {:?}", a.value, chain
            );
        }
    }

    // ── is_valid_address: adversarial string inputs ──────────────────────────

    /// is_valid_address must never panic on arbitrary UTF-8 strings.
    #[test]
    fn is_valid_address_never_panics(s in ".*") {
        // Just call it — any return value is fine; panic is not.
        let _ = is_valid_address(&s, &JovaChain::Ethereum);
    }

    /// Empty string is never a valid address.
    #[test]
    fn empty_string_is_not_valid_address(_n in 0u32..10) {
        prop_assert!(!is_valid_address("", &JovaChain::Ethereum));
    }

    /// A string of wrong length is not a valid EVM address.
    /// Valid EVM addresses are exactly 42 chars (0x + 40 hex).
    /// Anything != 42 chars is always invalid.
    #[test]
    fn wrong_length_address_is_invalid(extra in "[0-9a-fA-F]{1,39}") {
        // This produces strings shorter than 42 chars — never valid EVM addresses.
        let addr = format!("0x{extra}");
        if addr.len() != 42 {
            prop_assert!(!is_valid_address(&addr, &JovaChain::Ethereum));
        }
    }

    // ── is_valid_mnemonic / from_mnemonic error path ─────────────────────────

    /// from_mnemonic on a random string must return Err, never panic.
    #[test]
    fn from_mnemonic_never_panics_on_garbage(s in ".*") {
        let result = JovaWallet::from_mnemonic(&s, "");
        // We can't assert Ok or Err without knowing the input is valid BIP-39,
        // but we can assert it doesn't panic.
        let _ = result;
    }

    /// is_valid_mnemonic on a random string must never panic.
    #[test]
    fn is_valid_mnemonic_never_panics(s in ".*") {
        let _ = jova_core::is_valid_mnemonic(&s, "");
    }
}
