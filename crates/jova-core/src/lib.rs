//! jova-core — public Rust API.

#![forbid(unsafe_code)]

mod chain;
mod error;
mod wallet;

pub use chain::JovaChain;
pub use error::JovaError;
pub use jova_core_chains::{
    AccessListItem, Address, EvmUnsigned, SignableMessage, Signature, SignedTx, UnsignedTx,
};
pub use jova_core_primitives::Strength;
pub use wallet::{create_mnemonic, is_valid_address, is_valid_mnemonic, JovaWallet};
