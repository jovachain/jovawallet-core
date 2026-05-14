//! Solana raw ed25519 message signing.
//!
//! Solana does not define a canonical message-signing scheme equivalent to
//! BIP-322 / EIP-191 — there's no prefix, no chain-id binding, no length
//! envelope. The convention adopted by every major Solana wallet (Phantom,
//! Solflare, Backpack) is "sign the raw bytes the dApp provides".
//!
//! This module implements that convention: decode `message_base64`, sign
//! the raw bytes with ed25519, return the base58 signature.

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use jova_core_primitives::Ed25519Xprv;

use crate::error::ChainError;

/// Sign an arbitrary base64-encoded byte payload with the wallet's ed25519
/// key. Returns the 64-byte signature in base58 encoding (Solana convention).
pub fn sign_sol_message(
    xprv: &Ed25519Xprv,
    message_base64: &str,
) -> Result<String, ChainError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(message_base64)
        .map_err(|_| ChainError::MalformedSignableMessage("sol_invalid_base64".into()))?;
    let signing_key = SigningKey::from_bytes(xprv.secret_bytes());
    let sig = signing_key.sign(&bytes);
    Ok(bs58::encode(sig.to_bytes()).into_string())
}
