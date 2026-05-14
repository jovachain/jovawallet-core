//! Solana chain support: base58 ed25519 address derivation, VersionedTransaction
//! (v0) signing with ALT support, and raw ed25519 message signing.
//!
//! Solana uses the SLIP-10 ed25519 derivation path `m/44'/501'/0'/0'/0'`
//! (Phantom/Solflare convention; all components hardened — SLIP-10 ed25519
//! requirement). The derivation lives in
//! `jova_core_primitives::derive_ed25519` and produces an `Ed25519Xprv`.
//!
//! Because `Ed25519Xprv` is a distinct key type from the secp256k1 `XPrv`
//! consumed by `ChainSigner`, `SolSigner` is intentionally NOT a
//! `ChainSigner` implementation. It's a sibling type with methods that take
//! `&Ed25519Xprv`; the `JovaWallet` dispatch routes Solana variants directly
//! to `SolSigner` without going through the trait. See the commit message
//! and `wallet.rs` for the rationale (the trait stays secp256k1-only — the
//! alternative of either genericizing the trait or erasing the key type to
//! `&[u8; 32]` was deemed more invasive than this special-case routing).

pub mod address;
pub mod message;
pub mod tx;

pub use address::{derive_sol_address, validate_sol_address};
pub use message::sign_sol_message;
pub use tx::sign_sol_tx;

use jova_core_primitives::Ed25519Xprv;

use crate::address::{Address, Signature, SignedTx};
use crate::error::ChainError;
use crate::signable_message::SignableMessage;
use crate::unsigned_tx::UnsignedTx;

/// Solana signing facade.
///
/// **Not a `ChainSigner` implementation** — `ChainSigner` is locked to the
/// secp256k1 `XPrv` key type, and Solana uses an ed25519 leaf key
/// (`Ed25519Xprv`). `SolSigner`'s methods take `&Ed25519Xprv` directly;
/// `JovaWallet` special-cases dispatch for Solana, deriving via
/// `derive_ed25519` and calling these methods without going through the
/// trait. This is intentional: the alternatives — genericizing the trait
/// over the key type, or type-erasing the key to `&[u8; 32]` — were both
/// judged more invasive than this surgical routing.
pub struct SolSigner;

impl SolSigner {
    pub fn derive_address(&self, key: &Ed25519Xprv) -> Result<Address, ChainError> {
        Ok(Address {
            chain: "solana".to_string(),
            value: derive_sol_address(key)?,
        })
    }

    pub fn validate_address(&self, addr: &str) -> bool {
        validate_sol_address(addr)
    }

    pub fn sign_tx(
        &self,
        key: &Ed25519Xprv,
        unsigned: &UnsignedTx,
    ) -> Result<SignedTx, ChainError> {
        match unsigned {
            UnsignedTx::Solana {
                message_base64,
                recent_blockhash,
            } => {
                let (raw_hex, tx_hash) = sign_sol_tx(key, message_base64, recent_blockhash)?;
                Ok(SignedTx {
                    chain: "solana".to_string(),
                    raw_hex,
                    tx_hash,
                })
            }
            _ => Err(ChainError::MalformedUnsignedTx(
                "expected_solana_variant".into(),
            )),
        }
    }

    pub fn sign_message(
        &self,
        key: &Ed25519Xprv,
        msg: &SignableMessage,
    ) -> Result<Signature, ChainError> {
        match msg {
            SignableMessage::Solana { message_base64 } => {
                let sig_b58 = sign_sol_message(key, message_base64)?;
                Ok(Signature { hex: sig_b58 })
            }
            _ => Err(ChainError::MalformedSignableMessage(
                "expected_solana_message".into(),
            )),
        }
    }
}
