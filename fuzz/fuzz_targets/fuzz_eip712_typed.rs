#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{JovaWallet, SignableMessage};

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let msg = SignableMessage::EvmTypedDataV4 { json: s.to_string() };
        let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
        let _ = w.sign_message(&msg);
    }
});
