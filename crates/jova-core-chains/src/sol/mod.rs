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
pub mod tx;

pub use address::{derive_sol_address, validate_sol_address};
pub use tx::sign_sol_tx;
