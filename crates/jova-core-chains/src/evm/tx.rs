use crate::error::ChainError;
use crate::unsigned_tx::EvmUnsigned;
use jova_core_primitives::XPrv;

/// Sign an EIP-1559 fee-market transaction.
///
/// Returns `(raw_hex, tx_hash)` where:
/// - `raw_hex` is the EIP-2718 encoded signed transaction (`0x02…`)
/// - `tx_hash` is the keccak-256 of the encoded transaction (`0x…`)
pub fn sign(key: &XPrv, tx: &EvmUnsigned) -> Result<(String, String), ChainError> {
    use alloy::consensus::{SignableTransaction, TxEip1559};
    use alloy::eips::eip2930::{AccessList, AccessListItem as AlloyAcl};
    use alloy::primitives::{Address as AlloyAddress, Bytes, TxKind, B256, U256};
    use alloy::signers::local::LocalSigner;
    use alloy::signers::SignerSync;

    // 1. Parse fields.
    let to: AlloyAddress = tx
        .to
        .parse()
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_to_address_invalid".into()))?;

    let value = U256::from_str_radix(tx.value.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;
    let max_fee = U256::from_str_radix(tx.max_fee_per_gas.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;
    let max_pri = U256::from_str_radix(tx.max_priority_fee_per_gas.trim(), 10)
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_decimal_string_invalid".into()))?;

    let data_bytes = hex::decode(tx.data.trim_start_matches("0x"))
        .map_err(|_| ChainError::MalformedUnsignedTx("evm_data_not_hex".into()))?;
    let data: Bytes = data_bytes.into();

    // 2. Access list.
    let mut access_list_items = Vec::with_capacity(tx.access_list.len());
    for item in &tx.access_list {
        let address: AlloyAddress = item
            .address
            .parse()
            .map_err(|_| ChainError::MalformedUnsignedTx("evm_access_list_invalid".into()))?;
        let mut keys = Vec::with_capacity(item.storage_keys.len());
        for k in &item.storage_keys {
            let kb: B256 = k
                .parse()
                .map_err(|_| ChainError::MalformedUnsignedTx("evm_access_list_invalid".into()))?;
            keys.push(kb);
        }
        access_list_items.push(AlloyAcl { address, storage_keys: keys });
    }

    // 3. Build the typed transaction.
    //
    // Alloy 2.0 API notes vs plan:
    // - `chain_id` field type is `u64` (= `ChainId`), no cast needed beyond `as u64`
    // - `max_fee_per_gas` / `max_priority_fee_per_gas` are `u128`, not `U256`; use
    //   `try_into().unwrap_or(u128::MAX)` to convert from U256 as the plan specifies.
    // - `to` is `TxKind::Call(addr)` for a normal transfer/call.
    let alloy_tx = TxEip1559 {
        chain_id: tx.chain_id,
        nonce: tx.nonce,
        gas_limit: tx.gas_limit,
        max_fee_per_gas: max_fee.try_into().unwrap_or(u128::MAX),
        max_priority_fee_per_gas: max_pri.try_into().unwrap_or(u128::MAX),
        to: TxKind::Call(to),
        value,
        access_list: AccessList(access_list_items),
        input: data,
    };

    // 4. Sign.
    //
    // Alloy 2.0: `LocalSigner` is in `alloy::signers::local`; `SignerSync` provides
    // `sign_hash_sync`. `LocalSigner::from_bytes` accepts `&B256`.
    let signer = LocalSigner::from_bytes(&B256::from_slice(key.private_key_bytes()))
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;

    // `signature_hash()` is from the `SignableTransaction` trait.
    let sig_hash = alloy_tx.signature_hash();
    let signature = signer
        .sign_hash_sync(&sig_hash)
        .map_err(|_| ChainError::SigningFailed("secp256k1_signing_error".into()))?;

    // 5. Encode.
    //
    // Alloy 2.0: `into_signed` is on `SignableTransaction` trait; the returned
    // `Signed<TxEip1559>` exposes `eip2718_encode(&mut buf)` and `hash() -> &B256`.
    let signed = alloy_tx.into_signed(signature);
    let mut buf = Vec::new();
    signed.eip2718_encode(&mut buf);
    let raw_hex = format!("0x{}", hex::encode(&buf));
    // `signed.hash()` returns `&B256` in alloy 2.0 (lazy OnceCell). Dereference to copy.
    let tx_hash = format!("0x{}", hex::encode(*signed.hash()));
    Ok((raw_hex, tx_hash))
}
