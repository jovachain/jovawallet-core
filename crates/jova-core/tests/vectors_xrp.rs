//! Phase 3b: XRP vector parity tests.
//!
//! Loads `spec/test-vectors.json` and iterates every `xrp.*` vector,
//! exercising the Rust core through `JovaWallet` and asserting byte-for-byte
//! match against the reference values captured from `xrpl-py 4.5` +
//! `bip_utils 2.x` (BIP-44 coin type 144).
//!
//! Mirrors `vectors_btc.rs` (Phase 2). XRPL has no standard message-signing
//! scheme so there is no `xrp_sign_message_vectors` test; the error vector
//! coverage exercises the `MalformedUnsignedTx` reason vocabulary.

use jova_core::{JovaChain, JovaError, JovaWallet, UnsignedTx};
use serde_json::Value;

/// Load all vectors from the spec file (embedded at compile time so tests are hermetic).
fn load_vectors() -> Vec<Value> {
    let raw = include_str!("../../../spec/test-vectors.json");
    let v: Value = serde_json::from_str(raw).expect("spec/test-vectors.json is valid JSON");
    v["vectors"]
        .as_array()
        .expect("'vectors' array exists")
        .clone()
}

#[test]
fn xrp_address_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "address" {
            continue;
        }
        if v["input"]["chain"]["kind"] != "xrp" {
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
            .address(&JovaChain::Xrp, 0)
            .unwrap_or_else(|e| panic!("vector {id}: address() failed: {e}"));

        assert_eq!(got.value, expected, "vector {id}: XRP address mismatch");
        ran += 1;
    }
    assert!(
        ran >= 1,
        "expected at least 1 XRP address vector, ran {ran}"
    );
}

#[test]
fn xrp_sign_tx_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "sign_tx" {
            continue;
        }
        if v["input"]["unsigned_tx"]["kind"] != "xrp" {
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
            .sign_tx(&unsigned)
            .unwrap_or_else(|e| panic!("vector {id}: sign_tx() failed: {e}"));

        // xrpl-py emits uppercase hex; the SDK normalizes to uppercase too,
        // but compare case-insensitively to be forgiving of either side.
        assert_eq!(
            signed.raw_hex.to_uppercase(),
            expected_hex.to_uppercase(),
            "vector {id}: signed_hex mismatch — Rust output differs from xrpl-py reference"
        );
        assert_eq!(
            signed.tx_hash.to_uppercase(),
            expected_hash.to_uppercase(),
            "vector {id}: tx_hash mismatch — Rust output differs from xrpl-py reference"
        );
        ran += 1;
    }
    assert!(
        ran >= 2,
        "expected at least 2 XRP sign_tx vectors, ran {ran}"
    );
}

#[test]
fn xrp_error_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        let id = v["id"].as_str().unwrap_or("?");
        if v["kind"] != "error" {
            continue;
        }
        if !id.starts_with("xrp.") {
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

        // XRP error vectors all carry an `unsigned_tx` — XRPL has no message
        // signing surface, so there's no message-error variant.
        let result: Result<(), JovaError> = if v["input"].get("unsigned_tx").is_some() {
            let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
                .unwrap_or_else(|e| panic!("vector {id}: deserialise unsigned_tx: {e}"));
            wallet.sign_tx(&unsigned).map(|_| ())
        } else {
            panic!("XRP error vector {id} must carry an unsigned_tx in input");
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
            Err(other) => panic!("vector {id}: wrong error type, got: {other}"),
        }
        ran += 1;
    }
    assert!(ran >= 2, "expected at least 2 XRP error vectors, ran {ran}");
}
