//! SLIP-10 ed25519 hierarchical key derivation.
//!
//! Implements the SLIP-0010 (<https://github.com/satoshilabs/slips/blob/master/slip-0010.md>)
//! key-derivation function for the **ed25519 curve**, used by Solana and
//! other ed25519-based chains. ed25519 derivation differs from BIP-32
//! secp256k1 in two key ways:
//!
//! 1. The master-key HMAC uses the salt `"ed25519 seed"` (not `"Bitcoin seed"`).
//! 2. Only **hardened** derivation is defined — non-hardened (index < 2^31)
//!    components are rejected, because ed25519 lacks the additive group
//!    structure that lets a child public key be derived from a parent public
//!    key without the parent secret.
//!
//! The Solana default derivation path is `m/44'/501'/0'/0'/0'` (Phantom /
//! Solflare convention), which is all-hardened by construction. This helper
//! enforces that property: any path containing a non-hardened component
//! returns `Ed25519DeriveError::HardenedRequired`.
//!
//! The `slip-10` crate on crates.io advertises only secp256k1 / secp256r1
//! support (its README explicitly says "Implementation currently does not
//! support ed25519 curve"), so the algorithm is implemented in-crate. The
//! algorithm is small (one HMAC-SHA512 per path component) and has been
//! cross-checked against `bip_utils.Bip44Coins.SOLANA` — see
//! `crates/jova-core-primitives/tests/slip10.rs`.

use crate::seed::Seed;
use ed25519_dalek::SigningKey;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha512 = Hmac<Sha512>;

const ED25519_SEED_KEY: &[u8] = b"ed25519 seed";
const HARDENED_OFFSET: u32 = 0x8000_0000;

/// Extended ed25519 private key (secret + chain code), produced by SLIP-10
/// derivation. Not `Clone` — produced fresh per-call from a long-lived seed
/// and lives only for the duration of one signing call.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Ed25519Xprv {
    pub(crate) secret: [u8; 32],
    pub(crate) chain_code: [u8; 32],
}

impl Ed25519Xprv {
    /// 32-byte ed25519 secret scalar (the seed for `SigningKey::from_bytes`).
    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret
    }

    /// 32-byte ed25519 public key.
    ///
    /// Internally: build an `ed25519_dalek::SigningKey` from the secret bytes
    /// and serialize the corresponding `VerifyingKey`. The signing key is
    /// dropped at end-of-scope; the only side-effect is a single scalar
    /// multiplication on Curve25519.
    pub fn public_key(&self) -> [u8; 32] {
        let sk = SigningKey::from_bytes(&self.secret);
        sk.verifying_key().to_bytes()
    }

    /// Construct an `Ed25519Xprv` directly from raw scalar + chain code.
    ///
    /// **Use only in tests and test-vector harnesses.** Normal code should
    /// produce an `Ed25519Xprv` via [`derive_ed25519`].
    pub fn from_raw_secret_and_chain_code(secret: [u8; 32], chain_code: [u8; 32]) -> Self {
        Self { secret, chain_code }
    }
}

impl core::fmt::Debug for Ed25519Xprv {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Ed25519Xprv(<redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ed25519DeriveError {
    /// SLIP-10 derivation failed (HMAC key length, seed too short, etc.).
    Slip10,
    /// A non-hardened path component (index < 2^31) was supplied. ed25519
    /// SLIP-10 derivation is only defined for hardened components.
    HardenedRequired,
}

impl core::fmt::Display for Ed25519DeriveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Slip10 => f.write_str("SLIP-10 ed25519 derivation failed"),
            Self::HardenedRequired => {
                f.write_str("SLIP-10 ed25519 requires every path component to be hardened")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Ed25519DeriveError {}

/// Derive an ed25519 extended private key at `path_indices` from `seed`.
///
/// `path_indices` is a slice of raw u32 path components in standard BIP-32
/// encoding: hardened components are `0x8000_0000 | n`, non-hardened are `n`
/// directly. The standard Solana path `m/44'/501'/0'/0'/0'` corresponds to
/// `[0x8000_002C, 0x8000_01F5, 0x8000_0000, 0x8000_0000, 0x8000_0000]`.
///
/// **Every component must be hardened.** A non-hardened component (any index
/// below `0x8000_0000`) returns [`Ed25519DeriveError::HardenedRequired`]; the
/// ed25519 SLIP-10 spec doesn't define non-hardened derivation at all.
pub fn derive_ed25519(
    seed: &Seed,
    path_indices: &[u32],
) -> Result<Ed25519Xprv, Ed25519DeriveError> {
    // Master-key generation: HMAC-SHA512(salt = "ed25519 seed", data = seed).
    // IL becomes the master secret scalar; IR becomes the master chain code.
    let mut hmac =
        HmacSha512::new_from_slice(ED25519_SEED_KEY).map_err(|_| Ed25519DeriveError::Slip10)?;
    hmac.update(seed.as_bytes());
    let result = hmac.finalize().into_bytes();

    let mut secret: [u8; 32] = result[..32]
        .try_into()
        .map_err(|_| Ed25519DeriveError::Slip10)?;
    let mut chain_code: [u8; 32] = result[32..]
        .try_into()
        .map_err(|_| Ed25519DeriveError::Slip10)?;

    // Walk the path. Each step is HMAC-SHA512(chain_code, 0x00 || parent_key || ser32(i)).
    // ed25519 SLIP-10 requires every component to be hardened.
    for &index in path_indices {
        if index < HARDENED_OFFSET {
            return Err(Ed25519DeriveError::HardenedRequired);
        }
        let (new_secret, new_chain_code) = ckd_priv(&secret, &chain_code, index)?;
        secret = new_secret;
        chain_code = new_chain_code;
    }

    Ok(Ed25519Xprv { secret, chain_code })
}

fn ckd_priv(
    parent_key: &[u8; 32],
    parent_chain_code: &[u8; 32],
    index: u32,
) -> Result<([u8; 32], [u8; 32]), Ed25519DeriveError> {
    let mut hmac =
        HmacSha512::new_from_slice(parent_chain_code).map_err(|_| Ed25519DeriveError::Slip10)?;
    // Hardened (mandatory for ed25519): data = 0x00 || parent_key || ser32(index)
    hmac.update(&[0x00]);
    hmac.update(parent_key);
    hmac.update(&index.to_be_bytes());

    let result = hmac.finalize().into_bytes();
    let il: [u8; 32] = result[..32]
        .try_into()
        .map_err(|_| Ed25519DeriveError::Slip10)?;
    let child_chain_code: [u8; 32] = result[32..]
        .try_into()
        .map_err(|_| Ed25519DeriveError::Slip10)?;
    // ed25519 SLIP-10: the child secret IS IL (no group-order reduction). All
    // 32-byte values are valid secret scalars.
    Ok((il, child_chain_code))
}
