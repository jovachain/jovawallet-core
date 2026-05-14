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
    // Phase 3+ adds: Solana.
}
