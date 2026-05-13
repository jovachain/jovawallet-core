#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{JovaWallet, UnsignedTx};

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(unsigned) = serde_json::from_str::<UnsignedTx>(s) {
            let w = JovaWallet::from_mnemonic(SEED, "").unwrap();
            let _ = w.sign_tx(&unsigned);
        }
    }
});
