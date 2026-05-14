//! Phase 2 Bitcoin dispatch integration tests.
//!
//! Proves that `JovaWallet` routes `JovaChain::Bitcoin`, `UnsignedTx::Bitcoin`,
//! and `SignableMessage::Bitcoin` through `BtcSigner` end-to-end. Full vector
//! coverage lives under Task 7/8; this file just exercises the wiring.

use jova_core::{BtcMsgScheme, JovaChain, JovaWallet, SignableMessage, UnsignedTx};

/// BIP-39 standard test mnemonic; same value used by every BTC vector.
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn btc_address_dispatch() {
    let wallet = JovaWallet::from_mnemonic(MNEMONIC, "").unwrap();
    let addr = wallet.address(&JovaChain::Bitcoin, 0).unwrap();
    assert_eq!(addr.chain, "bitcoin");
    assert_eq!(addr.value, "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
}

#[test]
fn btc_sign_tx_dispatch_single_input() {
    let wallet = JovaWallet::from_mnemonic(MNEMONIC, "").unwrap();
    let psbt = include_str!("../../../tools/btc-vector-capture/captures/single_input.psbt.b64")
        .trim()
        .to_string();
    let expected =
        include_str!("../../../tools/btc-vector-capture/captures/single_input.signed_hex")
            .trim()
            .to_lowercase();
    let signed = wallet
        .sign_tx(&UnsignedTx::Bitcoin { psbt_base64: psbt })
        .unwrap();
    assert_eq!(signed.chain, "bitcoin");
    assert!(
        !signed.raw_hex.starts_with("psbt:"),
        "single-input wallet-owned PSBT must finalize"
    );
    assert_eq!(signed.raw_hex.to_lowercase(), expected);
}

#[test]
fn btc_sign_tx_dispatch_two_party_returns_psbt_prefix() {
    let wallet = JovaWallet::from_mnemonic(MNEMONIC, "").unwrap();
    let psbt = include_str!("../../../tools/btc-vector-capture/captures/two_party.psbt.b64")
        .trim()
        .to_string();
    let signed = wallet
        .sign_tx(&UnsignedTx::Bitcoin { psbt_base64: psbt })
        .unwrap();
    assert_eq!(signed.chain, "bitcoin");
    assert!(
        signed.raw_hex.starts_with("psbt:"),
        "multi-party PSBT must use psbt: prefix"
    );
    assert_eq!(signed.tx_hash, "");
}

#[test]
fn btc_sign_message_dispatch_bip322() {
    let wallet = JovaWallet::from_mnemonic(MNEMONIC, "").unwrap();
    let expected = include_str!("../../../tools/btc-vector-capture/captures/bip322_sig.txt").trim();
    let msg = SignableMessage::Bitcoin {
        message: "Hello, Jova".to_string(),
        address: "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string(),
        scheme: BtcMsgScheme::Bip322,
    };
    let sig = wallet.sign_message(&msg).unwrap();
    assert_eq!(sig.hex, expected);
}

#[test]
fn btc_validates_address() {
    use jova_core::is_valid_address;
    assert!(is_valid_address(
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        &JovaChain::Bitcoin
    ));
    assert!(!is_valid_address(
        "0x9858EfFD232B4033E47d90003D41EC34EcaEda94",
        &JovaChain::Bitcoin
    ));
    assert!(!is_valid_address("not-an-address", &JovaChain::Bitcoin));
}
