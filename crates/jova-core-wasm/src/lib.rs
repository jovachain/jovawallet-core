//! jova-core-wasm — wasm-bindgen bindings.
//!
//! Phase 0 stub.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}
