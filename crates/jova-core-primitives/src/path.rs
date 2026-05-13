use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationPath {
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    Empty,
    NotPrefixed,
    InvalidComponent(String),
    IndexOutOfRange,
}

impl core::fmt::Display for PathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => f.write_str("empty path"),
            Self::NotPrefixed => f.write_str("path must start with m/"),
            Self::InvalidComponent(_) => f.write_str("invalid component"),
            Self::IndexOutOfRange => f.write_str("index out of range"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PathError {}

const HARDENED_OFFSET: u32 = 0x8000_0000;

impl DerivationPath {
    pub fn parse(s: &str) -> Result<Self, PathError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(PathError::Empty);
        }
        if !s.starts_with("m/") && s != "m" {
            return Err(PathError::NotPrefixed);
        }

        let parts: Vec<&str> = if s == "m" {
            Vec::new()
        } else {
            s["m/".len()..].split('/').collect()
        };

        // A bare "m/44" (single non-empty segment, no sub-segments) is treated as a
        // malformed path per the test "rejects_malformed_path". The plan test asserts
        // `DerivationPath::parse("m/44").is_err()`. However, "m/44" is technically a
        // valid depth-1 path by BIP-32 (index 44, unhardened). The test comment says
        // "rejects_malformed_path" — the plan wants this to fail. Checking the plan
        // test more carefully: it only asserts three cases:
        //   - "m/44" → err
        //   - "xx"   → err
        //   - ""     → err
        // "m/44" without a hardening mark is ambiguous with the plan intent. The eth
        // canonical path is "m/44'/60'/0'/0/0" (5 components). A single-component path
        // without hardening is degenerate for wallet use but not syntactically invalid
        // per BIP-32. We follow the plan test as written: "m/44" must return Err.
        // Rationale: the only valid single-segment use cases all require hardening (e.g.
        // coin-type). A bare `m/N` without hardening at depth 1 is rejected as
        // insufficient for any jova derivation path.
        //
        // HOWEVER — the plan test also asserts that "m/44'/60'/0'/0/0" parses to 5
        // indices, and "m/44h/60h/0h/0/0" parses the same. Both have 5 components
        // separated by '/'. "m/44" has only 1 component. The rejection of "m/44"
        // is most naturally read as: a path must have AT LEAST 2 depth components
        // (purpose + coin type). We enforce minimum depth = 2 for non-master paths.
        if parts.len() == 1 {
            return Err(PathError::InvalidComponent(parts[0].to_string()));
        }

        let mut indices = Vec::with_capacity(parts.len());
        for part in parts {
            let (num, hardened) = if let Some(stripped) = part.strip_suffix('\'') {
                (stripped, true)
            } else if let Some(stripped) = part.strip_suffix('h') {
                (stripped, true)
            } else {
                (part, false)
            };
            let n: u32 = num
                .parse()
                .map_err(|_| PathError::InvalidComponent(part.to_string()))?;
            if n >= HARDENED_OFFSET {
                return Err(PathError::IndexOutOfRange);
            }
            indices.push(if hardened { n + HARDENED_OFFSET } else { n });
        }
        Ok(Self { indices })
    }
}
