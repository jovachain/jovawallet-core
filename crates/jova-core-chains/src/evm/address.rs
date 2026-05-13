use jova_core_primitives::{XPrv, keccak_address};
use sha3::{Digest, Keccak256};

/// Derive the EIP-55 checksum address for the given extended private key.
pub fn derive(key: &XPrv) -> String {
    let pk = key.public_key_uncompressed();
    let lower = keccak_address(&pk); // 40 hex chars, lowercase, no 0x prefix
    eip55_checksum(&lower)
}

/// Apply EIP-55 checksum encoding to a hex address string.
///
/// `lower` may be passed with or without "0x" prefix; a lowercase body is assumed.
/// The returned string always has a "0x" prefix.
pub fn eip55_checksum(lower: &str) -> String {
    let lower = lower.trim_start_matches("0x").to_ascii_lowercase();
    let mut h = Keccak256::new();
    h.update(lower.as_bytes());
    let digest = h.finalize();
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, ch) in lower.chars().enumerate() {
        if ch.is_ascii_alphabetic() {
            // nibble index in the keccak digest: byte i/2, upper nibble when i is even
            let nibble = digest[i / 2] >> (if i % 2 == 0 { 4 } else { 0 }) & 0xf;
            if nibble >= 8 {
                out.push(ch.to_ascii_uppercase());
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Validate an EVM address string.
///
/// - Must start with "0x" and have length 42.
/// - All characters after "0x" must be valid hex.
/// - All-lowercase and all-uppercase are accepted as legacy.
/// - Mixed-case must match EIP-55 checksum exactly.
pub fn validate(addr: &str) -> bool {
    if !addr.starts_with("0x") || addr.len() != 42 {
        return false;
    }
    let body = &addr[2..];
    if !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    // Mixed-case must verify as EIP-55.  All-lower or all-upper are legacy-accepted.
    let has_upper = body.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = body.chars().any(|c| c.is_ascii_lowercase());
    if has_upper && has_lower {
        let expected = eip55_checksum(&body.to_ascii_lowercase());
        addr == expected
    } else {
        true
    }
}
