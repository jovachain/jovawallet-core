//! Track 0: FFI-level private-key constructor smoke test (Rust side of the
//! uniffi surface; binding-language parity is in spec vectors).

use jova_core_ffi::{FfiError, JovaChain, JovaWallet};

#[test]
fn ffi_from_private_key_evm_ok() {
    let key = "4646464646464646464646464646464646464646464646464646464646464646";
    let wallet = JovaWallet::from_private_key(key.to_string(), JovaChain::Ethereum)
        .expect("valid key");
    let addr = wallet.address(JovaChain::Ethereum, 0).expect("address");
    assert_eq!(
        addr.value.to_lowercase(),
        "0x9d8a62f656a8d1615c1294fd71e9cfb3e4855a4f"
    );
}

#[test]
fn ffi_from_private_key_bad_hex_maps_to_invalid_private_key() {
    let err = JovaWallet::from_private_key("zz".to_string(), JovaChain::Ethereum)
        .unwrap_err();
    assert!(
        matches!(err, FfiError::InvalidPrivateKey { .. }),
        "got {err:?}"
    );
}
