//! BIP-32 hierarchical key derivation for secp256k1 keys.
//!
//! Implements the BIP-32 Child Key Derivation (CKD) functions directly using
//! HMAC-SHA512 and the secp256k1 C library, without the bip32 crate's `secp256k1`
//! feature (which requires k256). This keeps the dependency tree clean.

use crate::keys::XPrv;
use crate::path::DerivationPath;
use crate::seed::Seed;
use hmac::{Hmac, Mac};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

const BITCOIN_SEED_KEY: &[u8] = b"Bitcoin seed";
const HARDENED_OFFSET: u32 = 0x8000_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeriveError {
    Bip32,
}

impl core::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BIP-32 derivation failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeriveError {}

/// Derive a secp256k1 extended private key at `path` from `seed`.
///
/// Implements BIP-32 master key generation and child key derivation.
/// Returns the key material at the specified derivation path as `XPrv`.
pub fn derive_secp256k1(seed: &Seed, path: &DerivationPath) -> Result<XPrv, DeriveError> {
    // BIP-32 master key generation: HMAC-SHA512 with key "Bitcoin seed"
    let mut hmac = HmacSha512::new_from_slice(BITCOIN_SEED_KEY).expect("HMAC key length valid");
    hmac.update(seed.as_bytes());
    let result = hmac.finalize().into_bytes();

    let mut key_bytes: [u8; 32] = result[..32].try_into().expect("slice is 32 bytes");
    let mut chain_code: [u8; 32] = result[32..].try_into().expect("slice is 32 bytes");

    // Validate master key is a valid secp256k1 scalar
    secp256k1::SecretKey::from_byte_array(key_bytes).map_err(|_| DeriveError::Bip32)?;

    // Walk the derivation path, applying CKD_priv at each step
    for &index in &path.indices {
        let (new_key, new_chain_code) =
            ckd_priv(&key_bytes, &chain_code, index).map_err(|_| DeriveError::Bip32)?;
        key_bytes = new_key;
        chain_code = new_chain_code;
    }

    Ok(XPrv { key: key_bytes, chain_code })
}

/// BIP-32 CKD_priv: derive child private key from parent.
///
/// For hardened children (index >= 0x80000000): HMAC-SHA512(chain_code, 0x00 || key || index)
/// For normal children: HMAC-SHA512(chain_code, ser_P(point(key)) || index)
fn ckd_priv(
    parent_key: &[u8; 32],
    parent_chain_code: &[u8; 32],
    index: u32,
) -> Result<([u8; 32], [u8; 32]), ()> {
    let mut hmac =
        HmacSha512::new_from_slice(parent_chain_code).map_err(|_| ())?;

    if index >= HARDENED_OFFSET {
        // Hardened: data = 0x00 || parent_key || index_be
        hmac.update(&[0x00]);
        hmac.update(parent_key);
    } else {
        // Normal: data = compressed_public_key || index_be
        let secp = secp256k1::Secp256k1::signing_only();
        let sk = secp256k1::SecretKey::from_byte_array(*parent_key).map_err(|_| ())?;
        let pk = sk.public_key(&secp);
        hmac.update(&pk.serialize()); // 33-byte compressed
    }

    // Append 4-byte big-endian index
    hmac.update(&index.to_be_bytes());

    let result = hmac.finalize().into_bytes();
    let il: [u8; 32] = result[..32].try_into().map_err(|_| ())?;
    let child_chain_code: [u8; 32] = result[32..].try_into().map_err(|_| ())?;

    // child_key = (il + parent_key) mod n  (secp256k1 tweak)
    let parent_sk = secp256k1::SecretKey::from_byte_array(*parent_key).map_err(|_| ())?;
    let child_sk = parent_sk.add_tweak(&secp256k1::Scalar::from_be_bytes(il).map_err(|_| ())?).map_err(|_| ())?;

    let child_key: [u8; 32] = child_sk.secret_bytes();
    Ok((child_key, child_chain_code))
}

/// Compute the Ethereum address from an uncompressed secp256k1 public key.
///
/// Applies Keccak-256 to the 64-byte payload (the key without the 0x04 prefix),
/// then returns the last 20 bytes as a lowercase hex string (no "0x" prefix).
pub fn keccak_address(uncompressed_pubkey: &[u8; 65]) -> alloc::string::String {
    use sha3::{Digest, Keccak256};
    let mut h = Keccak256::new();
    h.update(&uncompressed_pubkey[1..]); // drop leading 0x04
    let digest = h.finalize();
    hex::encode(&digest[12..]) // last 20 bytes → 40 hex chars
}
