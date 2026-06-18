//! Track 0: private-key import vector parity. Loads spec/test-vectors.json and
//! runs every `private_key_address` / `private_key_sign_tx` vector through
//! `JovaWallet::from_private_key`, asserting byte-for-byte against the
//! captured reference values.

use jova_core::{JovaChain, JovaWallet, UnsignedTx};
use serde_json::Value;

fn load_vectors() -> Vec<Value> {
    let raw = include_str!("../../../spec/test-vectors.json");
    let v: Value = serde_json::from_str(raw).expect("spec/test-vectors.json is valid JSON");
    v["vectors"].as_array().expect("'vectors' array exists").clone()
}

#[test]
fn private_key_address_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "private_key_address" {
            continue;
        }
        let id = v["id"].as_str().unwrap_or("?");
        let pk = v["input"]["private_key"].as_str().expect("private_key");
        let chain: JovaChain = serde_json::from_value(v["input"]["chain"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: chain deserialise: {e}"));
        let expected = v["expected"]["address"].as_str().expect("expected.address");

        let wallet = JovaWallet::from_private_key(pk, &chain)
            .unwrap_or_else(|e| panic!("vector {id}: from_private_key failed: {e}"));
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
    assert!(ran >= 3, "expected at least 3 private_key_address vectors, ran {ran}");
}

#[test]
fn private_key_sign_tx_vectors() {
    let mut ran = 0usize;
    for v in load_vectors() {
        if v["kind"] != "private_key_sign_tx" {
            continue;
        }
        let id = v["id"].as_str().unwrap_or("?");
        let pk = v["input"]["private_key"].as_str().expect("private_key");
        let chain: JovaChain = serde_json::from_value(v["input"]["chain"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: chain deserialise: {e}"));
        let unsigned: UnsignedTx = serde_json::from_value(v["input"]["unsigned_tx"].clone())
            .unwrap_or_else(|e| panic!("vector {id}: unsigned_tx deserialise: {e}"));
        let expected_hex = v["expected"]["signed_hex"].as_str().expect("signed_hex");
        let expected_hash = v["expected"]["tx_hash"].as_str().expect("tx_hash");

        let wallet = JovaWallet::from_private_key(pk, &chain)
            .unwrap_or_else(|e| panic!("vector {id}: from_private_key failed: {e}"));
        let signed = wallet
            .sign_tx(&unsigned)
            .unwrap_or_else(|e| panic!("vector {id}: sign_tx() failed: {e}"));
        assert_eq!(
            signed.raw_hex.to_lowercase(),
            expected_hex.to_lowercase(),
            "vector {id}: signed_hex mismatch"
        );
        assert_eq!(
            signed.tx_hash.to_lowercase(),
            expected_hash.to_lowercase(),
            "vector {id}: tx_hash mismatch"
        );
        ran += 1;
    }
    assert!(ran >= 1, "expected at least 1 private_key_sign_tx vector, ran {ran}");
}
