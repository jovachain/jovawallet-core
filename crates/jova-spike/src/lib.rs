//! jova-spike — feasibility spike. Throwaway. Phase 0 starts from a clean slate.

#[cfg_attr(feature = "ffi", uniffi::export)]
pub fn ping() -> String {
    "pong".to_string()
}

#[cfg_attr(feature = "ffi", uniffi::export)]
pub fn ping_chains() -> String {
    #[cfg(feature = "chain-evm")]
    let _ = std::any::type_name::<alloy::consensus::TxEip1559>();

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

    // NOTE: xrpl 0.1.2 on crates.io has NO keypair/signing types.
    // xrpl::core::keypairs::Seed does not exist in this crate.
    // The only public type is XrplClient (a WebSocket client).
    // This is a critical finding for the feasibility report.
    #[cfg(feature = "chain-xrp")]
    let _ = std::any::type_name::<xrpl::XrplClient>();

    "chains-linked".to_string()
}

#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn ping_wasm() -> String {
    ping()
}
