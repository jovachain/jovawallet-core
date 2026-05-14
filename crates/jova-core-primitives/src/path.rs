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

    /// Build a BIP-84 native SegWit path: `m/84'/0'/account'/change/index`.
    ///
    /// Coin type is hardcoded to `0` (Bitcoin mainnet). The three caller-supplied
    /// components must be below the hardened offset; the helper applies hardening
    /// to `account` internally.
    pub fn bip84_path(account: u32, change: u32, index: u32) -> Result<Self, PathError> {
        if account >= HARDENED_OFFSET || change >= HARDENED_OFFSET || index >= HARDENED_OFFSET {
            return Err(PathError::IndexOutOfRange);
        }
        Ok(Self {
            indices: alloc::vec![
                84 + HARDENED_OFFSET,
                HARDENED_OFFSET, // BTC mainnet coin type (0 hardened)
                account + HARDENED_OFFSET,
                change,
                index,
            ],
        })
    }

    /// Build a generic BIP-44 path: `m/44'/coin_type'/account'/change/index`.
    ///
    /// All four caller-supplied components must be below the hardened offset
    /// (2^31). The helper applies hardening to `coin_type` and `account`
    /// internally; `change` and `index` remain unhardened per BIP-44.
    ///
    /// Used by XRP (`coin_type = 144`) and is the canonical helper for any
    /// secp256k1 BIP-44 chain. EVM chains have always used the parser
    /// directly, but `bip44_path(60, 0, 0, 0)` is equivalent to
    /// `parse("m/44'/60'/0'/0/0")`.
    pub fn bip44_path(
        coin_type: u32,
        account: u32,
        change: u32,
        index: u32,
    ) -> Result<Self, PathError> {
        if coin_type >= HARDENED_OFFSET
            || account >= HARDENED_OFFSET
            || change >= HARDENED_OFFSET
            || index >= HARDENED_OFFSET
        {
            return Err(PathError::IndexOutOfRange);
        }
        Ok(Self {
            indices: alloc::vec![
                44 + HARDENED_OFFSET,
                coin_type + HARDENED_OFFSET,
                account + HARDENED_OFFSET,
                change,
                index,
            ],
        })
    }
}
