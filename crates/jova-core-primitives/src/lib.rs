//! jova-core-primitives — no_std-clean cryptographic primitives.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod derive;
mod error;
mod keys;
mod mnemonic;
mod path;
#[cfg(feature = "external-rng")]
mod rng;
mod seed;
mod slip10;

pub use derive::{DeriveError, derive_secp256k1, keccak_address};
pub use error::MnemonicError;
pub use keys::XPrv;
pub use mnemonic::{Mnemonic, Strength};
pub use path::{DerivationPath, PathError};
#[cfg(feature = "external-rng")]
pub use rng::{JovaRng, RngError};
pub use seed::Seed;
pub use slip10::{Ed25519DeriveError, Ed25519Xprv, derive_ed25519};

// Phase 0 stub — keep until Phase 1 task that removes it.
#[deprecated(note = "Phase 0 stub; use Mnemonic::validate instead")]
pub fn is_valid_mnemonic_stub(words: &str, _passphrase: &str) -> bool {
    Mnemonic::validate(words, _passphrase).is_ok()
}
