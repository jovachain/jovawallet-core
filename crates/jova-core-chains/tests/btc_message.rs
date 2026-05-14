//! Bitcoin message-signing parity tests.
//!
//! The captured BIP-322 simple signature is produced by an INDEPENDENT Python
//! signer (in `tools/btc-vector-capture/messages.sh`) built from `embit` 0.8.0
//! primitives and CROSS-VERIFIED at capture time by the `bip322` PyPI package
//! (a Rust-backed BIP-322 verifier). The legacy signMessage capture is also
//! produced by the same Python script and self-verified via pubkey recovery.
//! If `sign_btc_message` produces a different base64 string, the SDK is wrong
//! — not the vector. Re-capture procedure: `tools/btc-vector-capture/messages.sh`.

use jova_core_chains::ChainError;
use jova_core_chains::btc::{BtcMsgScheme, sign_btc_message};
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const MESSAGE: &str = "Hello, Jova";

const EXPECTED_BIP322_SIG: &str =
    include_str!("../../../tools/btc-vector-capture/captures/bip322_sig.txt");
const EXPECTED_LEGACY_SIG: &str =
    include_str!("../../../tools/btc-vector-capture/captures/legacy_sig.txt");

fn xprv() -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").expect("path");
    derive_secp256k1(&seed, &path).expect("derive")
}

#[test]
fn bip322_signature_matches_reference() {
    let sig = sign_btc_message(&xprv(), MESSAGE, ADDRESS, BtcMsgScheme::Bip322)
        .expect("BIP-322 sign should succeed");
    assert_eq!(
        sig.trim(),
        EXPECTED_BIP322_SIG.trim(),
        "BIP-322 signature must match captured reference byte-for-byte",
    );
}

#[test]
fn legacy_signature_matches_reference() {
    let sig = sign_btc_message(&xprv(), MESSAGE, ADDRESS, BtcMsgScheme::Legacy)
        .expect("legacy sign should succeed");
    assert_eq!(
        sig.trim(),
        EXPECTED_LEGACY_SIG.trim(),
        "Legacy signature must match captured reference byte-for-byte",
    );
}

#[test]
fn rejects_address_not_owned_by_wallet() {
    // BIP-173 example P2WPKH; not derived from the abandon-about mnemonic.
    let foreign = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    let result = sign_btc_message(&xprv(), MESSAGE, foreign, BtcMsgScheme::Bip322);
    match result {
        Err(ChainError::MalformedSignableMessage(reason)) => {
            assert_eq!(reason, "btc_message_address_mismatch");
        }
        other => panic!(
            "expected MalformedSignableMessage(\"btc_message_address_mismatch\"), got {:?}",
            other,
        ),
    }
}
