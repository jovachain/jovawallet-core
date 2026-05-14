//! Solana address derivation + validation.
//!
//! A Solana address is the base58 encoding of the raw 32-byte ed25519 public
//! key (no version byte, no checksum — Solana addresses are not base58check).
//! Per <https://docs.solana.com/developing/programming-model/accounts#account-address>,
//! the encoding alphabet is the Bitcoin base58 alphabet.
//!
//! Address length on the wire ranges 32–44 characters depending on the
//! pubkey value (leading zero bytes drop a base58 digit each).

use jova_core_primitives::Ed25519Xprv;
use solana_pubkey::Pubkey;

use crate::error::ChainError;

/// Derive a Solana base58 address from the ed25519 leaf key produced by
/// SLIP-10 derivation at `m/44'/501'/0'/0'/0'`.
///
/// Internally: `Pubkey::new_from_array(xprv.public_key())` wraps the raw
/// pubkey, then `Display` emits its canonical base58 form.
pub fn derive_sol_address(xprv: &Ed25519Xprv) -> Result<String, ChainError> {
    let pk = Pubkey::new_from_array(xprv.public_key());
    Ok(pk.to_string())
}

/// Validate that `s` is a syntactically well-formed Solana address.
///
/// Rejects:
/// - Strings shorter than 32 base58 characters or longer than 44 (the
///   length envelope for a 32-byte payload in the Bitcoin base58 alphabet).
/// - Strings containing characters outside the base58 alphabet.
/// - Strings whose decoded payload is not exactly 32 bytes.
///
/// Returns `true` for any 32-byte base58 string, including the all-zero
/// pubkey (`"11111111111111111111111111111111"`, the system program) — that
/// matches Solana's on-chain semantics where every 32-byte value is a
/// syntactically valid address.
pub fn validate_sol_address(s: &str) -> bool {
    if s.len() < 32 || s.len() > 44 {
        return false;
    }
    s.parse::<Pubkey>().is_ok()
}
