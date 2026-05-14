//! BIP-84 P2WPKH address encoding (`bc1q…` bech32) for Bitcoin mainnet.

use core::str::FromStr;

use bitcoin::{Address, AddressType, CompressedPublicKey, Network, PublicKey};
use jova_core_primitives::XPrv;

use crate::error::ChainError;

/// Derive a P2WPKH (BIP-84 native SegWit) mainnet address from an `XPrv`.
///
/// The `XPrv` is expected to already be derived at the BIP-84 path
/// (`m/84'/0'/account'/change/index`). The returned string is the canonical
/// lowercase bech32 form with the `bc1q` prefix.
pub fn derive_p2wpkh(xprv: &XPrv) -> Result<String, ChainError> {
    let compressed = xprv.public_key_compressed();
    let pk = PublicKey::from_slice(&compressed)
        .map_err(|_| ChainError::SigningFailed("pubkey_invalid".into()))?;
    let cpk = CompressedPublicKey::try_from(pk)
        .map_err(|_| ChainError::SigningFailed("pubkey_compress_failed".into()))?;
    let address = Address::p2wpkh(&cpk, Network::Bitcoin);
    Ok(address.to_string())
}

/// Validate that `s` is a canonical Bitcoin mainnet P2WPKH (`bc1q…`) address.
///
/// v1 SDK accepts only BIP-84 addresses. Legacy P2PKH (`1…`), P2SH (`3…`), and
/// Taproot (`bc1p…`) are rejected. Testnet / signet / regtest addresses are
/// rejected. Returns `false` on any parse failure.
pub fn validate_btc_address(s: &str) -> bool {
    let parsed = match Address::from_str(s) {
        Ok(a) => a,
        Err(_) => return false,
    };
    let checked = match parsed.require_network(Network::Bitcoin) {
        Ok(a) => a,
        Err(_) => return false,
    };
    matches!(checked.address_type(), Some(AddressType::P2wpkh))
}
