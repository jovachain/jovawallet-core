//! Single-input PSBT signing for Bitcoin P2WPKH (BIP-84).
//!
//! `sign_psbt` decodes a base64 PSBT, identifies inputs whose `witness_utxo`
//! is a P2WPKH spending to our derived pubkey, computes the BIP-143 sighash,
//! ECDSA-signs (low-S, RFC 6979 deterministic), and inserts the partial
//! signature. For single-input P2WPKH that we fully own, the PSBT is then
//! finalized manually (witness = `[sig_der_with_sighash, compressed_pk]`) and
//! the consensus-serialized signed transaction is returned. For PSBTs with
//! inputs we cannot sign, the updated (still-unfinalized) PSBT is returned.
//!
//! References:
//! - BIP-143 (segwit sighash): <https://github.com/bitcoin/bips/blob/master/bip-0143.mediawiki>
//! - BIP-174 (PSBT):           <https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki>
//! - BIP-84 (native SegWit):   <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bitcoin::PublicKey;
use bitcoin::consensus::Encodable;
use bitcoin::ecdsa::Signature as EcdsaSignature;
use bitcoin::hashes::Hash;
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::{WPubkeyHash, Witness};
use jova_core_primitives::XPrv;

use crate::error::ChainError;

/// Result of attempting to sign a PSBT.
///
/// When the PSBT is fully signed and we can finalize it (e.g. single-input
/// P2WPKH that the supplied `XPrv` owns), `is_finalized` is `true`,
/// `tx_hex` holds the consensus-serialized signed transaction, and
/// `tx_hash` holds the transaction's `txid` as lowercase hex.
///
/// When the PSBT still has inputs we couldn't sign (multi-party flow),
/// `is_finalized` is `false` and `psbt_base64` holds the updated PSBT
/// with our partial signatures inserted, suitable for handing off to
/// another signer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PsbtSignResult {
    pub is_finalized: bool,
    pub tx_hex: Option<String>,
    pub psbt_base64: Option<String>,
    pub tx_hash: Option<String>,
}

/// Sign a PSBT with the given extended private key.
///
/// See module-level docs for behavior. Errors:
/// - `MalformedUnsignedTx("psbt_invalid_base64")`     — input not valid base64.
/// - `MalformedUnsignedTx("psbt_invalid_serialization")` — input not a valid PSBT.
/// - `MalformedUnsignedTx("psbt_no_signable_inputs")` — no input whose
///   `witness_utxo` is P2WPKH for our pubkey's hash.
/// - `SigningFailed(...)` on internal crypto failures (should be unreachable
///   given valid PSBT and valid `XPrv`).
pub fn sign_psbt(xprv: &XPrv, psbt_base64: &str) -> Result<PsbtSignResult, ChainError> {
    let bytes = BASE64
        .decode(psbt_base64.trim())
        .map_err(|_| ChainError::MalformedUnsignedTx("psbt_invalid_base64".into()))?;
    let mut psbt = Psbt::deserialize(&bytes)
        .map_err(|_| ChainError::MalformedUnsignedTx("psbt_invalid_serialization".into()))?;

    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("xprv_invalid_scalar".into()))?;
    let compressed_pk_bytes = xprv.public_key_compressed();
    let our_pk = PublicKey::from_slice(&compressed_pk_bytes)
        .map_err(|_| ChainError::SigningFailed("pubkey_invalid".into()))?;
    // hash160(compressed_pubkey) — must match the P2WPKH program for inputs we own.
    let our_wpkh = WPubkeyHash::hash(&compressed_pk_bytes);

    // First pass: compute sighashes for any input that is P2WPKH-for-us.
    // We do this before mutating partial_sigs because SighashCache borrows
    // the unsigned tx, which lives inside the PSBT.
    let inputs_len = psbt.inputs.len();
    let mut to_sign: Vec<(usize, [u8; 32], bitcoin::Amount, bitcoin::ScriptBuf)> = Vec::new();
    {
        let unsigned_tx = psbt.unsigned_tx.clone();
        let mut cache = SighashCache::new(&unsigned_tx);
        for index in 0..inputs_len {
            let input = &psbt.inputs[index];
            let Some(utxo) = input.witness_utxo.as_ref() else {
                continue;
            };
            let spk = &utxo.script_pubkey;
            if !spk.is_p2wpkh() {
                continue;
            }
            // is_p2wpkh guarantees `[OP_0, <20-byte push>]`; the 20 bytes start at offset 2.
            let spk_bytes = spk.as_bytes();
            if spk_bytes.len() != 22 {
                continue;
            }
            let prog: [u8; 20] = match spk_bytes[2..22].try_into() {
                Ok(b) => b,
                Err(_) => continue,
            };
            if prog != our_wpkh.to_byte_array() {
                continue;
            }
            let sighash = cache
                .p2wpkh_signature_hash(index, spk, utxo.value, EcdsaSighashType::All)
                .map_err(|e| ChainError::SigningFailed(format!("sighash_failed:{e}")))?;
            to_sign.push((index, sighash.to_byte_array(), utxo.value, spk.clone()));
        }
    }

    if to_sign.is_empty() {
        return Err(ChainError::MalformedUnsignedTx(
            "psbt_no_signable_inputs".into(),
        ));
    }

    // Sign each identified input. RFC 6979 deterministic + normalize_s gives
    // a canonical signature: byte-stable across implementations.
    for (index, sighash_bytes, _value, _spk) in &to_sign {
        let msg = Message::from_digest(*sighash_bytes);
        let mut sig = secp.sign_ecdsa(&msg, &sk);
        sig.normalize_s();
        let bitcoin_sig = EcdsaSignature {
            signature: sig,
            sighash_type: EcdsaSighashType::All,
        };
        psbt.inputs[*index].partial_sigs.insert(our_pk, bitcoin_sig);
    }

    // Decide whether we can fully finalize. A single-input PSBT that we
    // signed is the canonical Phase 2 case; multi-input/multi-party stays
    // unfinalized so the next signer can complete it.
    let can_finalize = inputs_len == 1 && to_sign.len() == 1;

    if !can_finalize {
        let updated = psbt.serialize();
        return Ok(PsbtSignResult {
            is_finalized: false,
            tx_hex: None,
            psbt_base64: Some(BASE64.encode(&updated)),
            tx_hash: None,
        });
    }

    // Manual P2WPKH finalization: witness = [sig_der || sighash_byte, compressed_pk].
    let (sig_pk, sig) = psbt.inputs[0]
        .partial_sigs
        .iter()
        .next()
        .map(|(k, v)| (*k, *v))
        .ok_or_else(|| ChainError::SigningFailed("missing_partial_sig".into()))?;
    debug_assert_eq!(sig_pk, our_pk);

    let mut witness = Witness::new();
    witness.push(sig.to_vec());
    witness.push(sig_pk.to_bytes());

    psbt.inputs[0].final_script_witness = Some(witness);
    psbt.inputs[0].partial_sigs.clear();
    psbt.inputs[0].sighash_type = None;
    psbt.inputs[0].bip32_derivation.clear();
    // witness_utxo can be kept; extract_tx_unchecked_fee_rate doesn't care.

    let tx = psbt.extract_tx_unchecked_fee_rate();

    let mut tx_bytes = Vec::new();
    tx.consensus_encode(&mut tx_bytes)
        .map_err(|e| ChainError::SigningFailed(format!("tx_encode_failed:{e}")))?;
    let tx_hex = hex::encode(&tx_bytes);
    let txid = tx.compute_txid();
    // Display impl prints big-endian (RPC/explorer form). Use that for tx_hash.
    let tx_hash = format!("{txid:x}");

    Ok(PsbtSignResult {
        is_finalized: true,
        tx_hex: Some(tx_hex),
        psbt_base64: None,
        tx_hash: Some(tx_hash),
    })
}
