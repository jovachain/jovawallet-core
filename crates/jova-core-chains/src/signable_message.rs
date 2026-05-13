use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SignableMessage {
    EvmPersonalSign { message: String },
    EvmTypedDataV4 { json: String },
    // Phase 2+ adds: Solana, Bitcoin.
}
