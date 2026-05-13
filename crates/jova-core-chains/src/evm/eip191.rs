use crate::error::ChainError;
use jova_core_primitives::XPrv;
use sha3::{Digest, Keccak256};

/// Sign a UTF-8 string with EIP-191 personal_sign prefix.
///
/// Returns `0x` + 65-byte signature (r || s || v) where v is 27 or 28.
///
/// Alloy 2.0 adaptation: we use `LocalSigner::sign_hash_sync` (from alloy's
/// `SignerSync` trait) and `Signature::as_bytes()` (which encodes v as 27 +
/// parity) instead of the secp256k1 crate's `sign_ecdsa_recoverable` method
/// (which requires the `recovery` feature not present in the workspace dep).
pub fn sign(key: &XPrv, message: &str) -> Result<String, ChainError> {
    // EIP-191 prefix: "\x19Ethereum Signed Message:\n" + decimal byte length
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut h = Keccak256::new();
    h.update(prefix.as_bytes());
    h.update(message.as_bytes());
    let digest = h.finalize();
    sign_hash(key, &digest)
}

/// Sign a pre-computed 32-byte hash with the secp256k1 private key via alloy.
///
/// Returns `0x` + 65-byte recoverable ECDSA signature (r || s || v).
/// `v` follows the EIP-191 convention: 27 + y_parity (i.e. 27 or 28).
///
/// Alloy's `Signature::as_bytes()` produces the canonical `[r(32) || s(32) || v(1)]`
/// layout with v = 27 + y_parity — exactly the EIP-191/EIP-712 format.
pub(crate) fn sign_hash(key: &XPrv, hash: &[u8]) -> Result<String, ChainError> {
    use alloy::primitives::B256;
    use alloy::signers::SignerSync;
    use alloy::signers::local::LocalSigner;

    let hash_b256: B256 = B256::from_slice(hash);
    let signer = LocalSigner::from_bytes(&B256::from_slice(key.private_key_bytes()))
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;
    let sig = signer
        .sign_hash_sync(&hash_b256)
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;

    // `as_bytes()` returns [r(32) || s(32) || v(1)] with v = 27 + y_parity.
    let bytes = sig.as_bytes();
    let hex_out = format!("0x{}", hex::encode(bytes));
    Ok(hex_out)
}
