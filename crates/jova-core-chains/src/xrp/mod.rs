//! XRP chain support: classic address (`r…`) derivation, canonical XRPL
//! binary serialization, secp256k1 signing with RFC-6979 + DER + SHA512Half.
//!
//! BIP-44 coin type 144. XRPL message signing is intentionally not supported
//! (no canonical scheme exists in the protocol).

pub mod address;

pub use address::{derive_xrp_address, validate_xrp_address};
