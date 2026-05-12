use crate::error::ChainError;
use jova_core_primitives::XPrv;

/// Sign EIP-712 v4 typed data JSON.
///
/// `json` must be a string-serialised TypedData object as defined in EIP-712.
/// Returns `0x` + 65-byte recoverable ECDSA signature (r || s || v).
pub fn sign_typed_data_v4(key: &XPrv, json: &str) -> Result<String, ChainError> {
    use alloy::dyn_abi::TypedData;

    let typed: TypedData = serde_json::from_str(json)
        .map_err(|_| ChainError::MalformedSignableMessage("eip712_typed_data_invalid_json".into()))?;
    let digest = typed
        .eip712_signing_hash()
        .map_err(|_| ChainError::MalformedSignableMessage("eip712_unknown_type".into()))?;

    super::eip191::sign_hash(key, digest.as_slice())
}
