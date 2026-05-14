//! Solana VersionedTransaction (v0) signing tests.
//!
//! Cross-checks `sign_sol_tx` byte-for-byte against the reference output
//! from `solders` (Python Solana SDK) at
//! `tools/sol-vector-capture/captures/system_transfer_v0.signed.json`.

use jova_core_chains::ChainError;
use jova_core_chains::sol::sign_sol_tx;
use jova_core_primitives::{Mnemonic, derive_ed25519};
use serde_json::Value;

const HARDENED: u32 = 0x8000_0000;
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn sol_path() -> [u32; 5] {
    [HARDENED | 44, HARDENED | 501, HARDENED, HARDENED, HARDENED]
}

fn load_capture(name: &str) -> Value {
    let raw = std::fs::read_to_string(format!(
        "../../tools/sol-vector-capture/captures/{name}.signed.json"
    ))
    .unwrap_or_else(|e| panic!("read {name} capture: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {name} capture: {e}"))
}

#[test]
fn sol_sign_system_transfer_matches_solders() {
    let cap = load_capture("system_transfer_v0");
    let message_base64 = cap["message_base64"].as_str().expect("message_base64");
    let recent_blockhash = cap["recent_blockhash"].as_str().expect("recent_blockhash");
    let expected_hex = cap["signed_hex"].as_str().expect("signed_hex");
    let expected_sig_b58 = cap["signature_b58"].as_str().expect("signature_b58");

    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");

    let (signed_hex, sig_b58) =
        sign_sol_tx(&xprv, message_base64, recent_blockhash).expect("sign_sol_tx");

    assert_eq!(
        signed_hex, expected_hex,
        "signed_hex mismatch vs solders reference"
    );
    assert_eq!(
        sig_b58, expected_sig_b58,
        "signature_b58 mismatch vs solders reference"
    );
}

#[test]
fn sol_sign_with_alt_matches_solders() {
    let cap = load_capture("with_alt_v0");
    let message_base64 = cap["message_base64"].as_str().expect("message_base64");
    let recent_blockhash = cap["recent_blockhash"].as_str().expect("recent_blockhash");
    let expected_hex = cap["signed_hex"].as_str().expect("signed_hex");
    let expected_sig_b58 = cap["signature_b58"].as_str().expect("signature_b58");

    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");

    let (signed_hex, sig_b58) =
        sign_sol_tx(&xprv, message_base64, recent_blockhash).expect("sign_sol_tx with ALT");
    assert_eq!(signed_hex, expected_hex, "ALT signed_hex mismatch");
    assert_eq!(sig_b58, expected_sig_b58, "ALT signature_b58 mismatch");
}

#[test]
fn sol_sign_rejects_invalid_base64() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");
    let err = sign_sol_tx(&xprv, "!!! not base64 !!!", "anything").expect_err("should fail");
    match err {
        ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "sol_invalid_base64");
        }
        other => panic!("wrong error variant: {other:?}"),
    }
}

#[test]
fn sol_sign_rejects_blockhash_mismatch() {
    let cap = load_capture("system_transfer_v0");
    let message_base64 = cap["message_base64"].as_str().expect("message_base64");
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");

    // Replace the supplied recent_blockhash with a different (also valid) one.
    let wrong_bh = "11111111111111111111111111111112";
    let err = sign_sol_tx(&xprv, message_base64, wrong_bh).expect_err("should fail");
    match err {
        ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "sol_blockhash_mismatch");
        }
        other => panic!("wrong error variant: {other:?}"),
    }
}

#[test]
fn sol_sign_rejects_garbage_message_bytes() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");
    // Valid base64 of garbage that won't deserialize as a VersionedMessage.
    let bad_b64 = "AAAAAAAA"; // 6 zero bytes
    let err = sign_sol_tx(&xprv, bad_b64, "anything").expect_err("should fail");
    match err {
        ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "sol_message_unsupported_version");
        }
        other => panic!("wrong error variant: {other:?}"),
    }
}
