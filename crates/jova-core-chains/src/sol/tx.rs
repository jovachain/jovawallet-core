//! Solana VersionedTransaction (v0) signing.
//!
//! Algorithm (per <https://docs.solana.com/developing/versioned-transactions>):
//!
//! 1. Decode `message_base64` → wire-form `VersionedMessage` bytes. For v0
//!    these start with a `0x80` version-marker byte; for legacy they start
//!    with the header (number-of-signatures byte) directly. Both are
//!    deserialized through the same `bincode::deserialize::<VersionedMessage>`
//!    path because the serde impl reads the prefix discriminant.
//! 2. Validate the supplied `recent_blockhash` matches the blockhash inside
//!    the deserialized message. The caller is responsible for paying the fee;
//!    a mismatch is a sign the caller fetched a stale value somewhere.
//! 3. Sign the wire bytes with ed25519 (the SDK key is at the SLIP-10 leaf
//!    `m/44'/501'/0'/0'/0'`). Solana signs the wire form including the `0x80`
//!    prefix — verified by cross-check against `solders.VersionedTransaction`.
//! 4. Construct a `VersionedTransaction { signatures: vec![sig], message }`
//!    with a single signature. bincode-serialize the transaction; the wire
//!    format is `short_vec(signatures) || versioned_message_wire`.
//! 5. Return `(hex(signed_bytes), base58(sig))`. The signature_b58 is, by
//!    Solana convention, the transaction's identifier (often called the
//!    `tx_hash` in cross-chain APIs even though Solana doesn't define a
//!    separate transaction hash — txs are addressed by their first signature).

use bincode::{deserialize as bincode_deserialize, serialize as bincode_serialize};
use ed25519_dalek::{Signer, SigningKey};
use jova_core_primitives::Ed25519Xprv;
use solana_message::VersionedMessage;
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;

use crate::error::ChainError;

/// Sign a Solana VersionedTransaction. Returns `(signed_hex, signature_b58)`.
///
/// The signature_b58 is used as the Solana transaction identifier (the
/// chain itself addresses transactions by their first signature; there is
/// no separate "tx hash" concept on Solana).
pub fn sign_sol_tx(
    xprv: &Ed25519Xprv,
    message_base64: &str,
    recent_blockhash: &str,
) -> Result<(String, String), ChainError> {
    // 1. base64 → bytes.
    let wire_bytes = base64_decode(message_base64)
        .map_err(|_| ChainError::MalformedUnsignedTx("sol_invalid_base64".into()))?;

    // 2. bincode-deserialize as VersionedMessage. Any prefix-byte error or
    // structural mismatch maps to `sol_message_unsupported_version`.
    let message: VersionedMessage = bincode_deserialize(&wire_bytes)
        .map_err(|_| ChainError::MalformedUnsignedTx("sol_message_unsupported_version".into()))?;

    // 3. Blockhash binding.
    let parsed_bh = message.recent_blockhash().to_string();
    if parsed_bh != recent_blockhash {
        return Err(ChainError::MalformedUnsignedTx(
            "sol_blockhash_mismatch".into(),
        ));
    }

    // 4. Sign the wire bytes with ed25519. (Solana signs the same wire form
    // that gets embedded into the VersionedTransaction — verified offline
    // against `solders.VersionedTransaction` during vector capture.)
    let signing_key = SigningKey::from_bytes(xprv.secret_bytes());
    let sig_dalek = signing_key.sign(&wire_bytes);
    let sig_bytes = sig_dalek.to_bytes();
    let sig = Signature::from(sig_bytes);

    // 5. Construct the VersionedTransaction and bincode-serialize.
    let tx = VersionedTransaction {
        signatures: vec![sig],
        message,
    };
    let signed_bytes = bincode_serialize(&tx)
        .map_err(|e| ChainError::SigningFailed(format!("sol_tx_serialize_failed:{e}")))?;

    let signed_hex = hex::encode(&signed_bytes);
    let signature_b58 = bs58::encode(sig_bytes).into_string();
    Ok((signed_hex, signature_b58))
}

/// Local base64 decode wrapper — workspace dep `base64 0.22` engines are
/// awkward to call inline. Kept private so callers can't accidentally swap
/// the standard alphabet for url-safe.
fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}
