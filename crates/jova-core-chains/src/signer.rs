use jova_core_primitives::XPrv;

use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::unsigned_tx::UnsignedTx;

pub trait ChainSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError>;
    fn validate_address(&self, addr: &str) -> bool;
    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError>;
    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError>;
}
