//! jova-core — public Rust API.
//!
//! Phase 0 stub. Phase 1 lands the full JovaWallet surface.

#![forbid(unsafe_code)]

pub use jova_core_primitives::is_valid_mnemonic_stub;

/// Phase 0 stub: validate a mnemonic. Real BIP-39 in Phase 1.
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    is_valid_mnemonic_stub(words, passphrase)
}
