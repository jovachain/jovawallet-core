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

    pub(crate) fn derivation_path(&self) -> &'static str {
        match self {
            Self::Bitcoin => "m/84'/0'/0'/0/0",
            Self::Xrp => "m/44'/144'/0'/0/0",
            // Phantom/Solflare convention; SLIP-10 ed25519 requires all-hardened.
            Self::Solana => "m/44'/501'/0'/0'/0'",
            // All EVM chains share m/44'/60'/0'/0/0 in v1; account index applied separately.
            _ => "m/44'/60'/0'/0/0",
        }
    }
}
