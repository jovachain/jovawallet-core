//! Bitcoin chain support: BIP-84 native SegWit addresses, BIP-174 PSBT signing,
//! and BIP-322 message signing.

pub mod address;
pub mod message;
pub mod psbt;

pub use address::{derive_p2wpkh, validate_btc_address};
pub use message::{BtcMsgScheme, sign_btc_message};
pub use psbt::{PsbtSignResult, sign_psbt};

use jova_core_primitives::XPrv;

use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;

/// `ChainSigner` implementation for Bitcoin (BIP-84 native SegWit).
///
/// Dispatches `sign_tx` to [`sign_psbt`] and `sign_message` to
/// [`sign_btc_message`]. Multi-party (unfinalized) PSBT results are returned
/// as `raw_hex = "psbt:" + base64`, with an empty `tx_hash`. Single-party
/// (wallet-owned) PSBTs return the finalized consensus-serialized tx hex.
pub struct BtcSigner;

impl ChainSigner for BtcSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError> {
        Ok(Address {
            chain: "bitcoin".to_string(),
            value: derive_p2wpkh(key)?,
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        validate_btc_address(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Bitcoin { psbt_base64 } => {
                let result = sign_psbt(key, psbt_base64)?;
                let raw = if result.is_finalized {
                    result.tx_hex.unwrap_or_default()
                } else {
                    format!("psbt:{}", result.psbt_base64.unwrap_or_default())
                };
                let hash = result.tx_hash.unwrap_or_default();
                Ok(SignedTx {
                    chain: "bitcoin".to_string(),
                    raw_hex: raw,
                    tx_hash: hash,
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx(
                "expected_bitcoin_variant".into(),
            )),
        }
    }

    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError> {
        match msg {
            SignableMessage::Bitcoin {
                message,
                address,
                scheme,
            } => {
                let hex = sign_btc_message(key, message, address, *scheme)?;
                Ok(Signature { hex })
            }
            _ => Err(ChainError::MalformedSignableMessage(
                "expected_bitcoin_message".into(),
            )),
        }
    }
}
