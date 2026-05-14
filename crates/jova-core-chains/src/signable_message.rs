use serde::{Deserialize, Serialize};

use crate::btc::BtcMsgScheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SignableMessage {
    EvmPersonalSign {
        message: String,
    },
    EvmTypedDataV4 {
        json: String,
    },
    Bitcoin {
        message: String,
        address: String,
        scheme: BtcMsgScheme,
    },
    /// Solana raw ed25519 message signing. Solana has no canonical message
    /// scheme (no BIP-322 / EIP-191 equivalent); the wallet convention is to
    /// sign arbitrary base64-encoded bytes directly with the ed25519 leaf
    /// key. Phantom, Solflare, and Backpack all do this.
    Solana {
        message_base64: String,
    },
}
