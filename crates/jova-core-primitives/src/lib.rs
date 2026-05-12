//! jova-core-primitives — no_std-clean cryptographic primitives.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod derive;
mod error;
mod keys;
mod mnemonic;
mod path;
mod seed;

pub use derive::{derive_secp256k1, keccak_address, DeriveError};
pub use error::MnemonicError;
pub use keys::XPrv;
pub use mnemonic::{Mnemonic, Strength};
pub use path::{DerivationPath, PathError};
pub use seed::Seed;

// Phase 0 stub — keep until Phase 1 task that removes it.
#[deprecated(note = "Phase 0 stub; use Mnemonic::validate instead")]
pub fn is_valid_mnemonic_stub(words: &str, _passphrase: &str) -> bool {
    Mnemonic::validate(words, _passphrase).is_ok()
}
