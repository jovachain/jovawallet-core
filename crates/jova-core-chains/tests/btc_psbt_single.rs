//! Single-input PSBT signing parity test.
//!
//! The captured PSBT + signed-tx-hex come from an INDEPENDENT signer (Python
//! `embit` 0.8.0) operating on the same BIP-84 test mnemonic. If `sign_psbt`
//! produces a different signed hex, the SDK is wrong — not the vector.
//! Capture procedure: `tools/btc-vector-capture/single_input.sh`.

use jova_core_chains::ChainError;
use jova_core_chains::btc::sign_psbt;
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const BIP84_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const UNSIGNED_PSBT_B64: &str =
    include_str!("../../../tools/btc-vector-capture/captures/single_input.psbt.b64");
const EXPECTED_SIGNED_HEX: &str =
    include_str!("../../../tools/btc-vector-capture/captures/single_input.signed_hex");

fn xprv_at(path_str: &str) -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse(path_str).expect("path");
    derive_secp256k1(&seed, &path).expect("derive")
}

#[test]
fn signs_single_input_psbt_to_finalized_tx() {
    let xprv = xprv_at("m/84'/0'/0'/0/0");
    let result = sign_psbt(&xprv, UNSIGNED_PSBT_B64.trim()).expect("signs");
    assert!(result.is_finalized, "expected finalized PSBT");
    assert_eq!(
        result.tx_hex.expect("tx_hex present").trim().to_lowercase(),
        EXPECTED_SIGNED_HEX.trim().to_lowercase(),
        "signed tx hex must match the captured reference byte-for-byte",
    );
    let tx_hash = result.tx_hash.expect("tx_hash present");
    // Sanity: txid is 64 lowercase hex chars.
    assert_eq!(tx_hash.len(), 64);
    assert!(tx_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(result.psbt_base64.is_none());
}

#[test]
fn rejects_non_base64_psbt() {
    let xprv = xprv_at("m/84'/0'/0'/0/0");
    let err = sign_psbt(&xprv, "not-base64!").expect_err("must fail");
    assert_eq!(
        err,
        ChainError::MalformedUnsignedTx("psbt_invalid_base64".into()),
    );
}

#[test]
fn rejects_valid_base64_but_invalid_psbt() {
    let xprv = xprv_at("m/84'/0'/0'/0/0");
    // "hello world" base64-encoded — valid base64, not a PSBT.
    let err = sign_psbt(&xprv, "aGVsbG8gd29ybGQ=").expect_err("must fail");
    assert_eq!(
        err,
        ChainError::MalformedUnsignedTx("psbt_invalid_serialization".into()),
    );
}

#[test]
fn rejects_psbt_with_no_signable_inputs() {
    // The Task 3 PSBT, but using a DIFFERENT mnemonic so our derived pubkey
    // doesn't match the witness program of any input.
    let foreign_mnemonic =
        "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic";
    let seed = Mnemonic::to_seed(foreign_mnemonic, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");

    let result = sign_psbt(&xprv, UNSIGNED_PSBT_B64.trim());
    match result {
        Err(ChainError::MalformedUnsignedTx(reason)) => {
            assert_eq!(reason, "psbt_no_signable_inputs");
        }
        other => panic!(
            "expected MalformedUnsignedTx(\"psbt_no_signable_inputs\"), got {:?}",
            other
        ),
    }
}
