#![no_main]
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use jova_core::{JovaWallet, UnsignedTx};
use libfuzzer_sys::fuzz_target;

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fuzz_target!(|data: &[u8]| {
    // Encode arbitrary bytes as base64 and pass as a PSBT. Should never panic;
    // should always return Err for non-PSBT inputs.
    let psbt_b64 = B64.encode(data);
    let unsigned = UnsignedTx::Bitcoin { psbt_base64: psbt_b64 };
    let wallet = JovaWallet::from_mnemonic(SEED, "").expect("static mnemonic is valid");
    let _ = wallet.sign_tx(&unsigned);
});
