//! XRPL transaction-signing tests. Each test cross-checks against an xrpl-py
//! capture (`tools/xrp-vector-capture/captures/*.signed.json`).

use jova_core_chains::xrp::sign_xrp_tx;
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};
use serde_json::Value;

const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn test_xprv() -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/44'/144'/0'/0/0").expect("path");
    derive_secp256k1(&seed, &path).expect("derive")
}

fn run_capture(capture_path: &str, capture: &str) {
    let captured: Value = serde_json::from_str(capture).expect("capture json");
    let tx_json = captured["tx_json"].clone();
    let expected_hex = captured["signed_hex"].as_str().expect("signed_hex");
    let expected_hash = captured["tx_hash"].as_str().expect("tx_hash");

    let tx_json_str = serde_json::to_string(&tx_json).expect("re-serialize tx_json");

    let xprv = test_xprv();
    let (signed_hex, tx_hash) = sign_xrp_tx(&xprv, &tx_json_str).expect("sign");

    assert_eq!(
        signed_hex.to_uppercase(),
        expected_hex.to_uppercase(),
        "signed_hex mismatch for capture {}",
        capture_path
    );
    assert_eq!(
        tx_hash.to_uppercase(),
        expected_hash.to_uppercase(),
        "tx_hash mismatch for capture {}",
        capture_path
    );
}

#[test]
fn signs_payment_with_destination_tag_matches_xrpl_py() {
    let capture = include_str!("../../../tools/xrp-vector-capture/captures/payment_dt.signed.json");
    run_capture("payment_dt.signed.json", capture);
}

#[test]
fn signs_offer_create_matches_xrpl_py() {
    let capture =
        include_str!("../../../tools/xrp-vector-capture/captures/offer_create.signed.json");
    run_capture("offer_create.signed.json", capture);
}

#[test]
fn rejects_non_json_input() {
    let xprv = test_xprv();
    let err = sign_xrp_tx(&xprv, "not-json").unwrap_err();
    match err {
        jova_core_chains::ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "xrp_invalid_json");
        }
        other => panic!("expected MalformedUnsignedTx, got {:?}", other),
    }
}

#[test]
fn rejects_non_object_json() {
    let xprv = test_xprv();
    let err = sign_xrp_tx(&xprv, "[1,2,3]").unwrap_err();
    assert!(matches!(
        err,
        jova_core_chains::ChainError::MalformedUnsignedTx(reason) if reason == "xrp_invalid_json"
    ));
}

#[test]
fn rejects_missing_transaction_type() {
    let xprv = test_xprv();
    let bad = r#"{"Account":"rHsMGQEkVNJmpGWs8XUBoTBiAAbwxZN5v3"}"#;
    let err = sign_xrp_tx(&xprv, bad).unwrap_err();
    match err {
        jova_core_chains::ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "xrp_missing_required_field:TransactionType");
        }
        other => panic!("expected MalformedUnsignedTx, got {:?}", other),
    }
}

#[test]
fn rejects_missing_account() {
    let xprv = test_xprv();
    let bad = r#"{"TransactionType":"Payment"}"#;
    let err = sign_xrp_tx(&xprv, bad).unwrap_err();
    match err {
        jova_core_chains::ChainError::MalformedUnsignedTx(reason) => {
            assert_eq!(reason, "xrp_missing_required_field:Account");
        }
        other => panic!("expected MalformedUnsignedTx, got {:?}", other),
    }
}
