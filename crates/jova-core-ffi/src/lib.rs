//! jova-core-ffi — uniffi-rs bindings layer.
//!
//! Phase 0 stub: re-export `is_valid_mnemonic`.

#![forbid(unsafe_code)]

#[uniffi::export]
pub fn is_valid_mnemonic(words: String, passphrase: String) -> bool {
    jova_core::is_valid_mnemonic(&words, &passphrase)
}

uniffi::setup_scaffolding!();
