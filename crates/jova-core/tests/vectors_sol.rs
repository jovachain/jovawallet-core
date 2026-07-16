//! Phase 3c: Solana vector parity tests.
//!
//! Loads `spec/test-vectors.json` and iterates every `sol.*` vector,
//! exercising the Rust core through `JovaWallet` and asserting byte-for-byte
//! match against the reference values captured from `solders 0.27` /
//! `bip_utils 2.x` / `PyNaCl`.
//!
//! Mirrors `vectors_xrp.rs`. The error-vector test covers both
//! `MalformedUnsignedTx` (sol.error.* with `unsigned_tx`) and
//! `MalformedSignableMessage` (sol.error.* with `message`) variants.

use jova_core::{JovaChain, JovaError, JovaWallet, SignableMessage, UnsignedTx};
use serde_json::Value;

fn load_vectors() -> Vec<Value> {
    let raw = include_str!("../../../spec/test-vectors.json");
    let v: Value = serde_json::from_str(raw).expect("spec/test-vectors.json is valid JSON");
    v["vectors"]
        .as_array()
        .expect("'vectors' array exists")
        .clone()
}

#[test]
fn sol_address_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "address" {
            continue;
        }
        if v["input"]["chain"]["kind"] != "solana" {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let pass = v["input"]["passphrase"].as_str().unwrap_or("");
        let expected = v["expected"]["address"]
            .as_str()
            .expect("expected.address exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let got = wallet
            .address(&JovaChain::Solana, 0)
            .unwrap_or_else(|e| panic!("vector {id}: address() failed: {e}"));

        assert_eq!(got.value, expected, "vector {id}: SOL address mismatch");
        ran += 1;
    }
    assert!(
        ran >= 1,
        "expected at least 1 SOL address vector, ran {ran}"
    );
}

#[test]
fn sol_sign_tx_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "sign_tx" {
            continue;
        }
        if v["input"]["unsigned_tx"]["kind"] != "solana" {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let pass = v["input"]["passphrase"].as_str().unwrap_or("");
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: deserialise unsigned_tx: {e}"));
        let expected_hex = v["expected"]["signed_hex"]
            .as_str()
            .expect("expected.signed_hex exists");
        let expected_hash = v["expected"]["tx_hash"]
            .as_str()
            .expect("expected.tx_hash exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let signed = wallet
            .sign_tx(&unsigned, 0)
            .unwrap_or_else(|e| panic!("vector {id}: sign_tx() failed: {e}"));

        assert_eq!(
            signed.raw_hex, expected_hex,
            "vector {id}: signed_hex mismatch — Rust output differs from solders reference"
        );
        assert_eq!(
            signed.tx_hash, expected_hash,
            "vector {id}: tx_hash (first-signature base58) mismatch"
        );
        ran += 1;
    }
    assert!(
        ran >= 2,
        "expected at least 2 SOL sign_tx vectors, ran {ran}"
    );
}

#[test]
fn sol_sign_message_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "sign_message" {
            continue;
        }
        if v["input"]["message"]["kind"] != "solana" {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let pass = v["input"]["passphrase"].as_str().unwrap_or("");
        let msg: SignableMessage = serde_json::from_value(v["input"]["message"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: deserialise message: {e}"));
        // signature_hex carries the base58 sig string by the Phase 1 convention
        // of using `signature_hex` regardless of encoding.
        let expected = v["expected"]["signature_hex"]
            .as_str()
            .expect("expected.signature_hex exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let sig = wallet
            .sign_message(&msg, 0)
            .unwrap_or_else(|e| panic!("vector {id}: sign_message() failed: {e}"));

        assert_eq!(sig.hex, expected, "vector {id}: signature mismatch");
        ran += 1;
    }
    assert!(
        ran >= 1,
        "expected at least 1 SOL sign_message vector, ran {ran}"
    );
}

#[test]
fn sol_error_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        let id = v["id"].as_str().unwrap_or("?");
        if v["kind"] != "error" {
            continue;
        }
        if !id.starts_with("sol.") {
            continue;
        }

        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let pass = v["input"]["passphrase"].as_str().unwrap_or("");
        let expected_variant = v["expected"]["error_variant"]
            .as_str()
            .expect("expected.error_variant exists");
        let expected_reason = v["expected"]["reason"]
            .as_str()
            .expect("expected.reason exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, pass)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));

        let result: Result<(), JovaError> = if v["input"].get("unsigned_tx").is_some() {
            let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
                .unwrap_or_else(|e| panic!("vector {id}: deserialise unsigned_tx: {e}"));
            wallet.sign_tx(&unsigned, 0).map(|_| ())
        } else if v["input"].get("message").is_some() {
            let msg: SignableMessage = serde_json::from_value(v["input"]["message"].clone())
                .unwrap_or_else(|e| panic!("vector {id}: deserialise message: {e}"));
            wallet.sign_message(&msg, 0).map(|_| ())
        } else {
            panic!("SOL error vector {id} must carry an unsigned_tx or message in input");
        };

        match result {
            Ok(_) => {
                panic!("vector {id}: expected error {expected_variant}/{expected_reason}, got Ok")
            }
            Err(JovaError::MalformedUnsignedTx { reason }) => {
                assert_eq!(
                    expected_variant, "MalformedUnsignedTx",
                    "vector {id}: wrong error variant (got MalformedUnsignedTx)"
                );
                assert_eq!(reason, expected_reason, "vector {id}: wrong error reason");
            }
            Err(JovaError::MalformedSignableMessage { reason }) => {
                assert_eq!(
                    expected_variant, "MalformedSignableMessage",
                    "vector {id}: wrong error variant (got MalformedSignableMessage)"
                );
                assert_eq!(reason, expected_reason, "vector {id}: wrong error reason");
            }
            Err(other) => panic!("vector {id}: wrong error type, got: {other}"),
        }
        ran += 1;
    }
    assert!(ran >= 4, "expected at least 4 SOL error vectors, ran {ran}");
}
