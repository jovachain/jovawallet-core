use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn ping_wasm() -> String {
    "pong-wasm".to_string()
}

#[wasm_bindgen]
pub fn ping_chains_wasm() -> String {
    // alloy is non-optional: pure-Rust k256 backend, WASM-safe.
    let _ = std::any::type_name::<alloy::consensus::TxEip1559>();

    // bitcoin + secp256k1 require secp256k1-sys (C FFI). On macOS, Apple clang
    // has no WASM backend, so these are gated behind chain-btc for the spike.
    #[cfg(feature = "chain-btc")]
    {
        let _ = std::any::type_name::<bitcoin::Address>();
        let _ = std::any::type_name::<bdk_wallet::Wallet>();
    }

    #[cfg(feature = "chain-sol")]
    {
        let _ = std::any::type_name::<solana_keypair::Keypair>();
        let _ = std::any::type_name::<solana_transaction::versioned::VersionedTransaction>();
    }

    // xrpl-rust 1.1 with features = ["core", "wallet", "models"]
    #[cfg(feature = "chain-xrp")]
    let _ = std::any::type_name::<xrpl::wallet::Wallet>();

    "chains-linked-wasm".to_string()
}
