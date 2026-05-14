use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum UnsignedTx {
    Evm(EvmUnsigned),
    Bitcoin {
        psbt_base64: String,
    },
    /// XRPL transaction: a JSON object (canonical XRPL field-naming) carrying
    /// the unsigned transaction. The signer injects `SigningPubKey` and
    /// `TxnSignature` itself; callers must NOT pre-populate those fields.
    Xrp {
        tx_json: String,
    },
    /// Solana VersionedTransaction (v0 or legacy): `message_base64` is the
    /// wire-form bincode-serialized `VersionedMessage` (v0 messages start
    /// with the 0x80 version-marker byte; legacy messages start with the
    /// number-of-signatures byte). `recent_blockhash` is the base58 form of
    /// the blockhash embedded in the message; the signer enforces a
    /// byte-equality check against the parsed message blockhash.
    Solana {
        message_base64: String,
        recent_blockhash: String,
    },
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
