//! XRPL transaction signing.
//!
//! Algorithm (per <https://xrpl.org/docs/references/protocol/transactions/common-fields>
//! and <https://xrpl.org/serialization.html>):
//!
//! 1. Parse the unsigned tx as a JSON object. Required fields:
//!    `TransactionType`, `Account`.
//! 2. Inject `SigningPubKey` = compressed secp256k1 pubkey hex (uppercase).
//! 3. `signing_bytes = encode_for_signing(tx)` — prepends the
//!    `0x53545800` (`"STX\0"`) signing prefix and serializes only
//!    `isSerialized && isSigningField` fields in canonical order.
//! 4. `digest = SHA512(signing_bytes)[..32]` (XRPL "SHA512Half").
//! 5. `sig = secp256k1.sign_ecdsa(digest, priv)`  — RFC-6979 deterministic +
//!    canonical low-S (libsecp256k1 default), DER-encoded.
//! 6. Inject `TxnSignature` = DER signature hex (uppercase).
//! 7. `final_hex = encode(tx)` — canonical signed binary, hex.
//! 8. `tx_hash = SHA512(b"TXN\0" || final_bytes)[..32]` (XRPL "TXN\0" prefix).
//!
//! The returned strings are uppercase hex to match xrpl-py / rippled
//! conventions and the spec vectors.

use jova_core_primitives::XPrv;
use secp256k1::{Message, Secp256k1, SecretKey, ecdsa::Signature};
use serde_json::Value;
use sha2::{Digest, Sha512};
use xrpl::core::binarycodec::{encode, encode_for_signing};

use crate::error::ChainError;

const TXN_HASH_PREFIX: &[u8; 4] = b"TXN\0";

/// Sign a canonical XRPL transaction. Input is a JSON string describing the
/// unsigned transaction. Output is `(signed_hex, tx_hash)` both uppercase hex.
pub fn sign_xrp_tx(xprv: &XPrv, tx_json: &str) -> Result<(String, String), ChainError> {
    // 1. Parse.
    let mut tx: Value = serde_json::from_str(tx_json)
        .map_err(|_| ChainError::MalformedUnsignedTx("xrp_invalid_json".into()))?;
    if !tx.is_object() {
        return Err(ChainError::MalformedUnsignedTx("xrp_invalid_json".into()));
    }

    // Required fields.
    for required in ["TransactionType", "Account"] {
        if tx.get(required).is_none() {
            return Err(ChainError::MalformedUnsignedTx(format!(
                "xrp_missing_required_field:{}",
                required
            )));
        }
    }

    // 2. Inject SigningPubKey.
    let pubkey = xprv.public_key_compressed();
    tx["SigningPubKey"] = Value::String(hex::encode_upper(pubkey));

    // 3. Canonical signing payload (xrpl-rust returns hex).
    let signing_hex = encode_for_signing(&tx)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_encode_for_signing:{:?}", e)))?;
    let signing_bytes = hex::decode(&signing_hex)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_encode_for_signing_hex:{}", e)))?;

    // 4. SHA512Half.
    let mut hasher = Sha512::new();
    hasher.update(&signing_bytes);
    let full = hasher.finalize();
    let mut digest_half = [0u8; 32];
    digest_half.copy_from_slice(&full[..32]);

    // 5. secp256k1 sign — RFC-6979 + canonical low-S (libsecp256k1 default),
    //    DER-encoded.
    let secp = Secp256k1::signing_only();
    let sk = SecretKey::from_byte_array(*xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("xrp_secret_invalid".into()))?;
    let msg = Message::from_digest(digest_half);
    let sig: Signature = secp.sign_ecdsa(msg, &sk);
    let der = sig.serialize_der();

    // 6. Inject TxnSignature.
    tx["TxnSignature"] = Value::String(hex::encode_upper(der.as_ref()));

    // 7. Final canonical signed binary.
    let final_hex =
        encode(&tx).map_err(|e| ChainError::SigningFailed(format!("xrp_encode:{:?}", e)))?;
    // xrpl-rust may emit lowercase; normalize to upper for byte-equality with
    // the xrpl-py spec vectors (xrpl-py emits uppercase).
    let final_hex_upper = final_hex.to_uppercase();
    let final_bytes = hex::decode(&final_hex_upper)
        .map_err(|e| ChainError::SigningFailed(format!("xrp_encode_hex:{}", e)))?;

    // 8. tx_hash = SHA512Half("TXN\0" || final_bytes).
    let mut h = Sha512::new();
    h.update(TXN_HASH_PREFIX);
    h.update(&final_bytes);
    let h_full = h.finalize();
    let tx_hash = hex::encode_upper(&h_full[..32]);

    Ok((final_hex_upper, tx_hash))
}
