//! jova-core-wasm — wasm-bindgen bindings.

#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}

#[wasm_bindgen(js_name = createMnemonic)]
pub fn create_mnemonic(bits256: bool) -> String {
    let s = if bits256 {
        jova_core::Strength::Bits256
    } else {
        jova_core::Strength::Bits128
    };
    let m = jova_core::create_mnemonic(s);
    m.words.clone()
}

// Phase 6 will add the full JovaWallet surface; for now we just confirm the crate compiles.
