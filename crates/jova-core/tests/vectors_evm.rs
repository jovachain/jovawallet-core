//! Phase 1 EVM vector tests.
//!
//! Loads `spec/test-vectors.json` and runs `JovaWallet`'s sign methods through
//! the same inputs, asserting byte-for-exact-byte match against the reference
//! values captured from `cast` (Foundry 1.7.1).

use jova_core::{JovaChain, JovaError, JovaWallet, SignableMessage, UnsignedTx};
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

// ── address vectors ──────────────────────────────────────────────────────────

#[test]
fn evm_address_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "address" {
            continue;
        }
        let chain_val = &v["input"]["chain"];
        let chain_kind = chain_val["kind"].as_str().expect("chain.kind exists");
        // Only EVM chains for Phase 1.
        if !matches!(
            chain_kind,
            "ethereum" | "polygon" | "bsc" | "arbitrum" | "optimism" | "base"
        ) {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
        let chain: JovaChain =
            serde_json::from_value(chain_val.clone()).expect("chain deserialises");
        let expected = v["expected"]["address"]
            .as_str()
            .expect("expected.address exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, passphrase)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let got = wallet
            .address(&chain, 0)
            .unwrap_or_else(|e| panic!("vector {id}: address() failed: {e}"));

        assert_eq!(
            got.value.to_lowercase(),
            expected.to_lowercase(),
            "vector {id}: address mismatch"
        );
        ran += 1;
    }
    assert!(ran >= 6, "expected at least 6 address vectors, ran {ran}");
}

// ── sign_tx vectors ──────────────────────────────────────────────────────────

#[test]
fn evm_sign_tx_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "sign_tx" {
            continue;
        }
        // Phase 2 added BTC sign_tx vectors with `unsigned_tx.kind == "bitcoin"`;
        // those are exercised by `vectors_btc.rs` in Task 8. Skip them here.
        if v["input"]["unsigned_tx"]["kind"] != "evm" {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: deserialise unsigned_tx: {e}"));
        let expected_hex = v["expected"]["signed_hex"]
            .as_str()
            .expect("expected.signed_hex exists");
        let expected_hash = v["expected"]["tx_hash"]
            .as_str()
            .expect("expected.tx_hash exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, passphrase)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let signed = wallet
            .sign_tx(&unsigned)
            .unwrap_or_else(|e| panic!("vector {id}: sign_tx() failed: {e}"));

        assert_eq!(
            signed.raw_hex.to_lowercase(),
            expected_hex.to_lowercase(),
            "vector {id}: signed_hex mismatch — Rust output differs from cast reference"
        );
        assert_eq!(
            signed.tx_hash.to_lowercase(),
            expected_hash.to_lowercase(),
            "vector {id}: tx_hash mismatch"
        );
        ran += 1;
    }
    assert!(ran >= 4, "expected at least 4 sign_tx vectors, ran {ran}");
}

// ── sign_message vectors ─────────────────────────────────────────────────────

#[test]
fn evm_sign_message_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "sign_message" {
            continue;
        }
        // Skip Phase 2 BTC sign_message vectors — they're exercised in `vectors_btc.rs`.
        let msg_kind = v["input"]["message"]["kind"].as_str().unwrap_or("");
        if !matches!(msg_kind, "evmPersonalSign" | "evmTypedDataV4") {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
        let msg: SignableMessage = serde_json::from_value(v["input"]["message"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: deserialise message: {e}"));
        let expected_sig = v["expected"]["signature"]
            .as_str()
            .expect("expected.signature exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, passphrase)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));
        let got = wallet
            .sign_message(&msg)
            .unwrap_or_else(|e| panic!("vector {id}: sign_message() failed: {e}"));

        assert_eq!(
            got.hex.to_lowercase(),
            expected_sig.to_lowercase(),
            "vector {id}: signature mismatch — Rust output differs from cast reference"
        );
        ran += 1;
    }
    assert!(
        ran >= 2,
        "expected at least 2 sign_message vectors, ran {ran}"
    );
}

// ── error vectors ────────────────────────────────────────────────────────────

#[test]
fn evm_error_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "error" {
            continue;
        }
        // Phase 1 error vectors all carry `input.unsigned_tx.kind == "evm"`;
        // Phase 2 added BTC error vectors (PSBT and signMessage paths). Filter
        // strictly so only EVM unsigned_tx error vectors land here.
        if v["input"]["unsigned_tx"]["kind"] != "evm" {
            continue;
        }

        let id = v["id"].as_str().unwrap_or("?");
        let mnemonic = v["input"]["mnemonic"].as_str().expect("mnemonic exists");
        let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
        let expected_variant = v["expected"]["error_variant"]
            .as_str()
            .expect("expected.error_variant exists");
        let expected_reason = v["expected"]["reason"]
            .as_str()
            .expect("expected.reason exists");

        let wallet = JovaWallet::from_mnemonic(mnemonic, passphrase)
            .unwrap_or_else(|e| panic!("vector {id}: from_mnemonic failed: {e}"));

        // All error vectors for Phase 1 exercise sign_tx with malformed input.
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: deserialise unsigned_tx: {e}"));

        let result = wallet.sign_tx(&unsigned);
        match result {
            Ok(_) => {
                panic!("vector {id}: expected error {expected_variant}/{expected_reason}, got Ok")
            }
            Err(JovaError::MalformedUnsignedTx { reason }) => {
                assert_eq!(
                    expected_variant, "MalformedUnsignedTx",
                    "vector {id}: wrong error variant"
                );
                assert_eq!(reason, expected_reason, "vector {id}: wrong error reason");
            }
            Err(other) => {
                panic!("vector {id}: wrong error type, got: {other}");
            }
        }
        ran += 1;
    }
    assert!(ran >= 3, "expected at least 3 error vectors, ran {ran}");
}
