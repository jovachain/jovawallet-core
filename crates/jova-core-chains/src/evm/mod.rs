mod address;
mod eip191;
mod eip712;
mod tx;

use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::signer::ChainSigner;
use crate::unsigned_tx::UnsignedTx;
use jova_core_primitives::XPrv;

/// Re-export address utilities for use in tests and jova-core.
pub use address::{derive as derive_address_str, eip55_checksum, validate as validate_address};

/// EVM chain signer.  Holds only a chain label string; all crypto state is
/// derived on-the-fly from the `XPrv` passed to each call.
pub struct EvmSigner {
    /// Short label used in `Address.chain` and `SignedTx.chain` output.
    pub chain_label: &'static str,
}

impl ChainSigner for EvmSigner {
    fn derive_address(&self, key: &XPrv) -> Result<Address, ChainError> {
        Ok(Address {
            chain: self.chain_label.to_string(),
            value: address::derive(key),
        })
    }

    fn validate_address(&self, addr: &str) -> bool {
        address::validate(addr)
    }

    fn sign_tx(&self, key: &XPrv, unsigned: &UnsignedTx) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Evm(evm) => {
                let (raw_hex, tx_hash) = tx::sign(key, evm)?;
                Ok(SignedTx {
                    chain: self.chain_label.to_string(),
                    raw_hex,
                    tx_hash,
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx(
                "expected_evm_variant".into(),
            )),
        }
    }

    fn sign_message(&self, key: &XPrv, msg: &SignableMessage) -> Result<Signature, ChainError> {
        let hex = match msg {
            SignableMessage::EvmPersonalSign { message } => eip191::sign(key, message)?,
            SignableMessage::EvmTypedDataV4 { json } => eip712::sign_typed_data_v4(key, json)?,
            _ => {
                return Err(ChainError::MalformedSignableMessage(
                    "expected_evm_message".into(),
                ));
            }
        };
        Ok(Signature { hex })
    }
}

/// Test helpers that expose internal construction for integration tests.
///
/// These are gated behind `#[cfg(test)]` on the _test binary_ side, but the
/// functions themselves must be accessible from the `tests/` directory (which
/// is a separate crate compilation unit).  We therefore compile them in all
/// modes and rely on the `test_helpers` module name to make it clear they
/// should never be called from production code.
pub mod test_helpers {
    use jova_core_primitives::XPrv;

    /// Build an `XPrv` from raw private-key bytes.
    ///
    /// # Safety / Warning
    /// This bypasses normal BIP-32 derivation.  Only for test use.
    pub fn xprv_from_key_bytes(key: [u8; 32]) -> XPrv {
        XPrv::from_raw_key_and_chain_code(key, [0u8; 32])
    }
}
