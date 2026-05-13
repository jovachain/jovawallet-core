//! jova-core — public Rust API.

#![forbid(unsafe_code)]

// Always-available surface: mnemonic primitives only.
pub use jova_core_primitives::{Mnemonic, Strength};

/// Generate a new random mnemonic with the given bit strength.
pub fn create_mnemonic(strength: Strength) -> Mnemonic {
    jova_core_primitives::Mnemonic::generate(strength)
}

/// Return `true` if `words` is a valid BIP-39 mnemonic with correct checksum.
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core_primitives::Mnemonic::validate(words, passphrase).is_ok()
}

// Full signing surface — gated on the `chains` feature so WASM builds
// (which can't compile secp256k1-sys) can use `default-features = false`.
#[cfg(feature = "chains")]
mod chain;
#[cfg(feature = "chains")]
mod error;
#[cfg(feature = "chains")]
mod wallet;

#[cfg(feature = "chains")]
pub use chain::JovaChain;
#[cfg(feature = "chains")]
pub use error::JovaError;
#[cfg(feature = "chains")]
pub use jova_core_chains::{
    AccessListItem, Address, EvmUnsigned, SignableMessage, Signature, SignedTx, UnsignedTx,
};
#[cfg(feature = "chains")]
pub use wallet::{JovaWallet, is_valid_address};
