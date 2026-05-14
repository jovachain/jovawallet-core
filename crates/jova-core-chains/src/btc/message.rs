//! Bitcoin message signing — BIP-322 simple and legacy `signMessage`.
//!
//! `sign_btc_message` derives the wallet's BIP-84 P2WPKH address from the
//! supplied `XPrv` and rejects any request to sign for a different address
//! (returns `ChainError::MalformedSignableMessage("btc_message_address_mismatch")`).
//! Otherwise it dispatches to the requested scheme.
//!
//! ## BIP-322 simple (modern, witness-based)
//!
//! Implemented in-tree against `bitcoin 0.32` per the BIP-322 spec
//! (<https://github.com/bitcoin/bips/blob/master/bip-0322.mediawiki>). No
//! external `bip322` Rust crate is pulled in (keeps engine confinement
//! tight, avoids version skew on `bitcoin` between deps).
//!
//! Algorithm:
//!
//! 1. `message_hash = BIP340_tagged_hash("BIP0322-signed-message", message)`.
//! 2. Build `to_spend` virtual tx: version=0, locktime=0, one input with
//!    prevout `(0x00..00, 0xFFFFFFFF)`, sequence=0, scriptSig =
//!    `OP_0 PUSH32 <message_hash>`; one output with value=0 and scriptPubKey
//!    = the address P2WPKH.
//! 3. Build `to_sign` virtual tx: version=0, locktime=0, one input spending
//!    `(to_spend_txid, 0)` with sequence=0; one output with value=0 and
//!    scriptPubKey = `OP_RETURN`.
//! 4. Compute the BIP-143 sighash on `to_sign` input 0 with `SIGHASH_ALL`
//!    against the witness-utxo from `to_spend`.
//! 5. Sign with secp256k1 **low-R** ECDSA (matches PSBT signing in this SDK
//!    and Bitcoin Core ≥ 0.17 / bdk / embit defaults — byte-stable output).
//! 6. Build witness `[sig_der || sighash_byte, compressed_pubkey]`.
//! 7. Output = base64 of the witness consensus-serialized bytes (i.e. the
//!    "bare" form ecosystem wallets like Sparrow, Unisat, ord, Leather
//!    produce — no `smp` variant prefix).
//!
//! ## Legacy `signMessage` (Bitcoin Core pre-BIP-322)
//!
//! Even though Bitcoin Core's `signmessage` RPC only ever supported P2PKH
//! addresses, the ecosystem (Electrum, hardware wallets, etc.) reuses the
//! same scheme for P2WPKH addresses by associating the recoverable
//! signature with the address conceptually. This SDK does the same.
//!
//! Algorithm:
//!
//! 1. `payload = "\x18Bitcoin Signed Message:\n" || varint(len(msg)) || msg`.
//! 2. `digest = sha256d(payload)`.
//! 3. Recoverable ECDSA sign (no low-R grinding — matches Bitcoin Core's
//!    `signmessage` path, which doesn't grind).
//! 4. `header = recovery_id + 31` (= recid + 27 + 4, where +4 marks the
//!    pubkey as compressed; BIP-84 keys are always compressed).
//! 5. Output = base64(`header || r(32) || s(32)`).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bitcoin::Witness;
use bitcoin::absolute::LockTime;
use bitcoin::consensus::Encodable;
use bitcoin::ecdsa::Signature as EcdsaSignature;
use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitcoin::secp256k1::ecdsa::RecoveryId;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::transaction::Version;
use bitcoin::{
    Address, AddressType, Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
};
use core::str::FromStr;
use jova_core_primitives::XPrv;
use serde::{Deserialize, Serialize};

use crate::btc::address::derive_p2wpkh;
use crate::error::ChainError;

/// Bitcoin message-signing scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BtcMsgScheme {
    /// BIP-322 simple (witness-based, modern). Returns
    /// `base64(witness_consensus_encoded)`.
    Bip322,
    /// Legacy `signMessage` (pre-BIP-322). Returns `base64(header || r || s)`,
    /// 65 raw bytes total before base64.
    Legacy,
}

/// Sign a UTF-8 message for the wallet's BIP-84 P2WPKH address.
///
/// `address` must equal the address derived from `xprv` (i.e. the wallet's
/// own P2WPKH address); foreign-address signing is rejected with
/// `MalformedSignableMessage("btc_message_address_mismatch")`.
///
/// Returns a base64-encoded signature whose interior format depends on
/// `scheme` (see module docs).
pub fn sign_btc_message(
    xprv: &XPrv,
    message: &str,
    address: &str,
    scheme: BtcMsgScheme,
) -> Result<String, ChainError> {
    // Reject any address that isn't this wallet's own derived P2WPKH.
    let derived = derive_p2wpkh(xprv)?;
    if derived != address {
        return Err(ChainError::MalformedSignableMessage(
            "btc_message_address_mismatch".into(),
        ));
    }

    match scheme {
        BtcMsgScheme::Bip322 => sign_bip322_simple(xprv, message, address),
        BtcMsgScheme::Legacy => sign_legacy(xprv, message),
    }
}

/// BIP-340 tagged hash: `SHA256(SHA256(tag) || SHA256(tag) || msg)`.
fn bip340_tagged_hash(tag: &[u8], msg: &[u8]) -> [u8; 32] {
    let tag_hash = sha256::Hash::hash(tag);
    let mut engine = sha256::Hash::engine();
    engine.input(tag_hash.as_byte_array());
    engine.input(tag_hash.as_byte_array());
    engine.input(msg);
    sha256::Hash::from_engine(engine).to_byte_array()
}

fn sign_bip322_simple(xprv: &XPrv, message: &str, address: &str) -> Result<String, ChainError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("xprv_invalid_scalar".into()))?;
    let compressed_pk_bytes = xprv.public_key_compressed();
    let pk = bitcoin::secp256k1::PublicKey::from_slice(&compressed_pk_bytes)
        .map_err(|_| ChainError::SigningFailed("pubkey_invalid".into()))?;

    // address must be a Bitcoin-mainnet P2WPKH (callers can only reach here
    // when derived == address, but defend against future surprises).
    let parsed = Address::from_str(address)
        .map_err(|_| ChainError::MalformedSignableMessage("btc_message_address_invalid".into()))?
        .require_network(Network::Bitcoin)
        .map_err(|_| {
            ChainError::MalformedSignableMessage("btc_message_address_wrong_network".into())
        })?;
    if parsed.address_type() != Some(AddressType::P2wpkh) {
        return Err(ChainError::MalformedSignableMessage(
            "btc_message_address_not_p2wpkh".into(),
        ));
    }
    let address_spk: ScriptBuf = parsed.script_pubkey();

    // Step 1: tagged hash of the message.
    let message_hash = bip340_tagged_hash(b"BIP0322-signed-message", message.as_bytes());

    // Step 2: to_spend.
    //   scriptSig = OP_0 PUSH32 <message_hash>
    //   OP_0 = 0x00, PUSHBYTES_32 opcode = 0x20.
    let mut script_sig_bytes = Vec::with_capacity(2 + 32);
    script_sig_bytes.push(0x00);
    script_sig_bytes.push(0x20);
    script_sig_bytes.extend_from_slice(&message_hash);
    let to_spend_script_sig = ScriptBuf::from_bytes(script_sig_bytes);

    let to_spend = Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            // OutPoint::null() == { txid: all-zeros, vout: u32::MAX } — exactly
            // the BIP-322 prevout requirement.
            previous_output: OutPoint::null(),
            script_sig: to_spend_script_sig,
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::ZERO,
            script_pubkey: address_spk.clone(),
        }],
    };
    let to_spend_txid = to_spend.compute_txid();

    // Step 3: to_sign.
    let op_return_spk = ScriptBuf::from_bytes(vec![0x6au8]); // bare OP_RETURN
    let to_sign = Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: to_spend_txid,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::ZERO,
            script_pubkey: op_return_spk,
        }],
    };

    // Step 4: BIP-143 sighash with SIGHASH_ALL over to_sign input 0, using the
    // to_spend output as the witness-utxo (value=0, spk=address P2WPKH).
    let sighash_bytes = {
        let mut cache = SighashCache::new(&to_sign);
        let sh = cache
            .p2wpkh_signature_hash(0, &address_spk, Amount::ZERO, EcdsaSighashType::All)
            .map_err(|e| ChainError::SigningFailed(format!("bip322_sighash_failed:{e}")))?;
        sh.to_byte_array()
    };

    // Step 5: low-R ECDSA sign. normalize_s for canonical low-S. Same low-R
    // grinding the PSBT signer uses — byte-stable against bdk/embit/bitcoind.
    let msg = Message::from_digest(sighash_bytes);
    let mut sig = secp.sign_ecdsa_low_r(&msg, &sk);
    sig.normalize_s();
    let bitcoin_sig = EcdsaSignature {
        signature: sig,
        sighash_type: EcdsaSighashType::All,
    };

    // Step 6: build witness [sig_der_with_sighash, compressed_pk].
    let mut witness = Witness::new();
    witness.push(bitcoin_sig.to_vec());
    witness.push(pk.serialize());

    // Step 7: base64 of the witness consensus encoding (the "bare" form
    // ecosystem wallets use; no `smp` variant prefix).
    let mut witness_bytes = Vec::new();
    witness
        .consensus_encode(&mut witness_bytes)
        .map_err(|e| ChainError::SigningFailed(format!("bip322_witness_encode_failed:{e}")))?;
    Ok(BASE64.encode(&witness_bytes))
}

fn sign_legacy(xprv: &XPrv, message: &str) -> Result<String, ChainError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(xprv.private_key_bytes())
        .map_err(|_| ChainError::SigningFailed("xprv_invalid_scalar".into()))?;

    // digest = sha256d(prefix || varint(len) || msg).
    // bitcoin::sign_message::signed_msg_hash does exactly this.
    let msg_hash = bitcoin::sign_message::signed_msg_hash(message);
    let msg = Message::from_digest(msg_hash.to_byte_array());

    // Recoverable ECDSA. Bitcoin Core's signmessage path doesn't grind for
    // low-R, so we match that here (no `sign_ecdsa_recoverable_low_r` exists
    // in secp256k1 0.29 anyway). The captured reference is produced the same
    // way, so byte-stability is preserved.
    let recoverable = secp.sign_ecdsa_recoverable(&msg, &sk);
    let (recid, compact): (RecoveryId, [u8; 64]) = recoverable.serialize_compact();

    // header_byte = recid + 27 + 4 (the +4 marks the recovered pubkey as
    // compressed, which all BIP-84 keys are).
    let recid_i32 = recid.to_i32();
    if !(0..=3).contains(&recid_i32) {
        return Err(ChainError::SigningFailed(format!(
            "legacy_recid_out_of_range:{recid_i32}",
        )));
    }
    let header: u8 = 27 + 4 + recid_i32 as u8;

    let mut bytes = [0u8; 65];
    bytes[0] = header;
    bytes[1..].copy_from_slice(&compact);
    Ok(BASE64.encode(bytes))
}
