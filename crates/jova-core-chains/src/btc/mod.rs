//! Bitcoin chain support: BIP-84 native SegWit addresses, BIP-174 PSBT signing,
//! and BIP-322 message signing.

pub mod address;

pub use address::{derive_p2wpkh, validate_btc_address};
