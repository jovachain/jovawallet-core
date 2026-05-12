//! Phase 0 hello-world: load the spec vector file and assert one
//! negative-validation vector returns false.

use serde_json::Value;

#[test]
fn vector_negative_mnemonic_validation() {
    let raw = include_str!("../../../spec/test-vectors.json");
    let vectors: Value = serde_json::from_str(raw).expect("vectors parse");
    let arr = vectors["vectors"].as_array().expect("vectors array");

    let v = arr
        .iter()
        .find(|v| v["id"] == "phase0.mnemonic_validation_neg.gibberish")
        .expect("phase0 vector present");

    let words = v["input"]["words"].as_str().expect("words");
    let passphrase = v["input"]["passphrase"].as_str().unwrap_or("");
    let expected = v["expected"]["valid"].as_bool().expect("expected.valid");

    assert_eq!(jova_core::is_valid_mnemonic(words, passphrase), expected);
}
