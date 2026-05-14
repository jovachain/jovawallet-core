//! XRP classic address (`r…`) derivation and validation.
//!
//! Algorithm (per <https://xrpl.org/cryptographic-keys.html#account-id-and-address>):
//!
//! 1. `pubkey` = SEC1-compressed secp256k1 public key (33 bytes).
//! 2. `account_id` = RIPEMD160(SHA256(pubkey))                  // 20 bytes
//! 3. `classic_address` = XRPL-base58check(0x00 || account_id)
//!
//! The encoding/validation are delegated to `xrpl-rust 1.1`'s
//! `core::addresscodec` module which implements the XRPL alphabet
//! (`rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz`) and the
//! 4-byte double-SHA256 checksum.

use jova_core_primitives::XPrv;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use xrpl::core::addresscodec::{encode_classic_address, is_valid_classic_address};

use crate::error::ChainError;

/// Derive an XRP classic address (`r…`) from an `XPrv` derived at the
/// BIP-44 XRP path (`m/44'/144'/0'/0/0`).
pub fn derive_xrp_address(xprv: &XPrv) -> Result<String, ChainError> {
    let pubkey = xprv.public_key_compressed();
    let account_id = account_id_from_pubkey(&pubkey);
    encode_classic_address(&account_id)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_address_encode_failed:{:?}", e)))
}

/// Validate that `s` is a syntactically and check-valid XRP classic address.
///
/// Rejects any address whose checksum doesn't verify, whose decoded version
/// byte isn't `0x00`, or whose length is wrong. Always rejects non-XRP
/// formats (EVM `0x…`, BTC `bc1q…`, etc.).
pub fn validate_xrp_address(s: &str) -> bool {
    is_valid_classic_address(s)
}

/// Compute the 20-byte XRPL Account ID from a compressed secp256k1 pubkey.
/// Exposed for use by the transaction-signing module's tx-hash routine.
pub(crate) fn account_id_from_pubkey(pubkey_compressed: &[u8; 33]) -> [u8; 20] {
    let sha = Sha256::digest(pubkey_compressed);
    let rip = Ripemd160::digest(sha);
    let mut out = [0u8; 20];
    out.copy_from_slice(&rip);
    out
}
