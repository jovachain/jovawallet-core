//! jova-core-chains — per-chain encoding and signing.

#![forbid(unsafe_code)]

pub mod evm;

mod address;
mod error;
mod signable_message;
mod signer;
mod unsigned_tx;

pub use address::{Address, Signature, SignedTx};
pub use error::ChainError;
pub use evm::EvmSigner;
pub use signable_message::SignableMessage;
pub use signer::ChainSigner;
pub use unsigned_tx::{AccessListItem, EvmUnsigned, UnsignedTx};
