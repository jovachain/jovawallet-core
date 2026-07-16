use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum JovaChain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
    Bitcoin,
    Xrp,
    Solana,
    CustomEvm { chain_id: u64 },
}

impl JovaChain {
    pub(crate) fn evm_chain_id(&self) -> Option<u64> {
        match self {
            Self::Ethereum => Some(1),
            Self::Polygon => Some(137),
            Self::Bsc => Some(56),
            Self::Arbitrum => Some(42161),
            Self::Optimism => Some(10),
            Self::Base => Some(8453),
            Self::CustomEvm { chain_id } => Some(*chain_id),
            Self::Bitcoin | Self::Xrp | Self::Solana => None,
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Bsc => "bsc",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::Bitcoin => "bitcoin",
            Self::Xrp => "xrp",
            Self::Solana => "solana",
            Self::CustomEvm { .. } => "customEvm",
        }
    }

    /// The BIP-32 / SLIP-10 derivation path for this chain at HD account
    /// index `account`.
    ///
    /// **Scheme:** `account` increments the BIP-44/84 `address_index` (the
    /// last path component) for the secp256k1 chains — identical to
    /// MetaMask's `m/44'/60'/0'/0/N`. Solana is the exception: SLIP-10
    /// ed25519 requires every component to be hardened, so `account`
    /// increments the hardened `account'` level (`m/44'/501'/N'/0'/0'`).
    /// `account = 0` reproduces the exact v0.4.0 paths byte-for-byte.
    pub(crate) fn derivation_path(&self, account: u32) -> String {
        match self {
            Self::Bitcoin => format!("m/84'/0'/0'/0/{account}"),
            Self::Xrp => format!("m/44'/144'/0'/0/{account}"),
            // SLIP-10 ed25519 requires all-hardened; increment the account'
            // level. The trailing /0'/0' preserves the v0.4.0 account-0 path.
            Self::Solana => format!("m/44'/501'/{account}'/0'/0'"),
            // All EVM chains share m/44'/60'/0'/0/N (MetaMask address_index).
            _ => format!("m/44'/60'/0'/0/{account}"),
        }
    }
}
