//! XRP chain support: classic address (`r…`) derivation, canonical XRPL
//! binary serialization, secp256k1 signing with RFC-6979 + DER + SHA512Half.
//!
//! BIP-44 coin type 144. XRPL message signing is intentionally not supported
//! (no canonical scheme exists in the protocol).

pub mod address;
pub mod tx;

pub use address::{derive_xrp_address, validate_xrp_address};
pub use tx::sign_xrp_tx;

use jova_core_primitives::XPrv;

use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;

/// `ChainSigner` implementation for XRP.
///
/// Address derivation: BIP-44 secp256k1 leaf → SHA256 → RIPEMD160 →
/// XRPL base58check classic address (`r…`).
///
/// Transaction signing: canonical XRPL binary serialization
/// (`xrpl-rust::core::binarycodec`), SHA512Half digest, RFC-6979 +
/// canonical-low-S ECDSA over secp256k1, DER signature embedded in the
/// signed payload, SHA512Half(`"TXN\0"` || final_bytes) as the tx_hash.
///
/// Message signing is intentionally unsupported — there is no canonical
/// XRPL message-signing scheme equivalent to BIP-322 / EIP-191.
pub struct XrpSigner;

impl ChainSigner for XrpSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError> {
        Ok(Address {
            chain: "xrp".to_string(),
            value: derive_xrp_address(key)?,
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        validate_xrp_address(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Xrp { tx_json } => {
                let (raw_hex, tx_hash) = sign_xrp_tx(key, tx_json)?;
                Ok(SignedTx {
                    chain: "xrp".to_string(),
                    raw_hex,
                    tx_hash,
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx(
                "expected_xrp_variant".into(),
            )),
        }
    }

    fn sign_message(&self, _key: &XPrv, _msg: &SignableMessage) -> Result<Signature, ChainError> {
        Err(ChainError::MalformedSignableMessage(
            "xrp_message_signing_unsupported".into(),
        ))
    }
}
