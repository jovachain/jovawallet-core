//! Solana raw message signing tests.

use jova_core_chains::sol::sign_sol_message;
use jova_core_chains::ChainError;
use jova_core_primitives::{Mnemonic, derive_ed25519};
use serde_json::Value;

const HARDENED: u32 = 0x8000_0000;
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn sol_path() -> [u32; 5] {
    [
        HARDENED | 44,
        HARDENED | 501,
        HARDENED,
        HARDENED,
        HARDENED,
    ]
}

#[test]
fn sol_sign_message_matches_solders() {
    // Reference captured by tools/sol-vector-capture/capture-tx.sh using
    // PyNaCl SigningKey(seed).sign(...).
    let raw = std::fs::read_to_string(
        "../../tools/sol-vector-capture/captures/sign_message.signed.json",
    )
    .expect("read sign_message capture");
    let cap: Value = serde_json::from_str(&raw).expect("parse capture");

    let message_base64 = cap["message_base64"].as_str().expect("message_base64");
    let expected_sig = cap["signature_b58"].as_str().expect("signature_b58");

    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");

    let sig = sign_sol_message(&xprv, message_base64).expect("sign_sol_message");
    assert_eq!(sig, expected_sig, "raw-message signature mismatch vs PyNaCl reference");
}

#[test]
fn sol_sign_message_rejects_invalid_base64() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");
    let err = sign_sol_message(&xprv, "$$$ not base64 $$$").expect_err("should fail");
    match err {
        ChainError::MalformedSignableMessage(reason) => {
            assert_eq!(reason, "sol_invalid_base64");
        }
        other => panic!("wrong error variant: {other:?}"),
    }
}

#[test]
fn sol_sign_message_handles_empty_payload() {
    // base64("") == "" (legitimate empty payload, signs as empty bytes).
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");
    let sig = sign_sol_message(&xprv, "").expect("sign empty");
    // 64-byte ed25519 sig base58-encodes to 86–88 chars.
    assert!(sig.len() >= 86 && sig.len() <= 88, "unexpected sig length: {}", sig.len());
}
