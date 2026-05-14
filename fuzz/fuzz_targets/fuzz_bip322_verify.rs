#![no_main]
// NB: filename retained per the Phase 2 Task 9 plan ("verify"), but this SDK
// only exposes the sign side of BIP-322 (verification is out-of-scope for v1).
// So this target fuzzes the sign path: arbitrary message bytes through both
// the BIP-322 and the legacy `signMessage` schemes, with a fixed address.
use jova_core::{BtcMsgScheme, JovaWallet, SignableMessage};
use libfuzzer_sys::fuzz_target;

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

fuzz_target!(|data: &[u8]| {
    if let Ok(message) = std::str::from_utf8(data) {
        let wallet = JovaWallet::from_mnemonic(SEED, "").expect("static mnemonic is valid");

        // BIP-322 path
        let msg322 = SignableMessage::Bitcoin {
            message: message.to_string(),
            address: ADDRESS.to_string(),
            scheme: BtcMsgScheme::Bip322,
        };
        let _ = wallet.sign_message(&msg322);

        // Legacy path
        let msglegacy = SignableMessage::Bitcoin {
            message: message.to_string(),
            address: ADDRESS.to_string(),
            scheme: BtcMsgScheme::Legacy,
        };
        let _ = wallet.sign_message(&msglegacy);
    }
});
