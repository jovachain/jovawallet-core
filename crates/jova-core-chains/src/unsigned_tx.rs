use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UnsignedTx {
    Evm(EvmUnsigned),
    Bitcoin { psbt_base64: String },
    /// XRPL transaction: a JSON object (canonical XRPL field-naming) carrying
    /// the unsigned transaction. The signer injects `SigningPubKey` and
    /// `TxnSignature` itself; callers must NOT pre-populate those fields.
    Xrp { tx_json: String },
    // Phase 3c adds: Solana.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmUnsigned {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: String,
    pub value: String, // wei, decimal string
    pub gas_limit: u64,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub data: String, // 0x-prefixed hex
    #[serde(default)]
    pub access_list: Vec<AccessListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListItem {
    pub address: String,
    pub storage_keys: Vec<String>,
}
