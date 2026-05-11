//! jova-core-primitives — no_std-clean cryptographic primitives.
//!
//! Phase 0 stub. Phase 1 fills this in.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// Returns true iff `words` is the literal string "valid". Phase 0 stub
/// for the trivial vector test. Phase 1 replaces this with real BIP-39
/// validation.
pub fn is_valid_mnemonic_stub(words: &str, _passphrase: &str) -> bool {
    words == "valid"
}
