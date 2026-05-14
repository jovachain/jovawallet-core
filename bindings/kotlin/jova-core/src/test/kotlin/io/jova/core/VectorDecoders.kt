package io.jova.core

import org.json.JSONObject
import uniffi.jova_core_ffi.AccessListItem
import uniffi.jova_core_ffi.BtcMsgScheme
import uniffi.jova_core_ffi.EvmUnsigned
import uniffi.jova_core_ffi.JovaChain
import uniffi.jova_core_ffi.SignableMessage
import uniffi.jova_core_ffi.UnsignedTx

class VectorDecodeException(msg: String) : RuntimeException(msg)

fun decodeChain(o: JSONObject): JovaChain = when (val kind = o.getString("kind")) {
    "ethereum" -> JovaChain.Ethereum
    "polygon"  -> JovaChain.Polygon
    "bsc"      -> JovaChain.Bsc
    "arbitrum" -> JovaChain.Arbitrum
    "optimism" -> JovaChain.Optimism
    "base"     -> JovaChain.Base
    "bitcoin"  -> JovaChain.Bitcoin
    "solana"   -> JovaChain.Solana
    "xrp"      -> JovaChain.Xrp
    "customEvm" -> JovaChain.CustomEvm(chainId = o.getLong("chainId").toULong())
    else       -> throw VectorDecodeException("unknown chain kind: $kind")
}

fun decodeEvmUnsigned(o: JSONObject): EvmUnsigned {
    val accessList = if (o.has("accessList")) {
        val arr = o.getJSONArray("accessList")
        (0 until arr.length()).map { i ->
            val item = arr.getJSONObject(i)
            val keysArr = item.getJSONArray("storage_keys")
            AccessListItem(
                address = item.getString("address"),
                storageKeys = (0 until keysArr.length()).map { keysArr.getString(it) }
            )
        }
    } else emptyList()
    return EvmUnsigned(
        chainId = o.getLong("chainId").toULong(),
        nonce = o.getLong("nonce").toULong(),
        to = o.getString("to"),
        value = o.getString("value"),
        gasLimit = o.getLong("gasLimit").toULong(),
        maxFeePerGas = o.getString("maxFeePerGas"),
        maxPriorityFeePerGas = o.getString("maxPriorityFeePerGas"),
        `data` = o.getString("data"),
        accessList = accessList
    )
}

/**
 * Decode an `input.unsigned_tx` JSON object into the uniffi `UnsignedTx` enum.
 * Routes by the `kind` discriminator: `"evm"`, `"bitcoin"`, and `"xrp"` are
 * supported. Future kinds (e.g. `"solana"`) throw `VectorDecodeException`
 * (callers `continue` past them).
 */
fun decodeUnsignedTx(o: JSONObject): UnsignedTx = when (val kind = o.getString("kind")) {
    "evm"     -> UnsignedTx.Evm(tx = decodeEvmUnsigned(o))
    "bitcoin" -> UnsignedTx.Bitcoin(psbtBase64 = o.getString("psbt_base64"))
    "xrp"     -> UnsignedTx.Xrp(txJson = o.getString("tx_json"))
    "solana"  -> UnsignedTx.Solana(
        messageBase64    = o.getString("message_base64"),
        recentBlockhash  = o.getString("recent_blockhash")
    )
    else      -> throw VectorDecodeException("unknown unsigned_tx kind: $kind")
}

/**
 * Map the spec's camelCase `scheme` field to the uniffi `BtcMsgScheme` enum.
 * uniffi 0.31 emits SCREAMING_SNAKE for enum variants, so `bip322` → `BIP322`.
 */
private fun decodeBtcMsgScheme(s: String): BtcMsgScheme = when (s) {
    "bip322" -> BtcMsgScheme.BIP322
    "legacy" -> BtcMsgScheme.LEGACY
    else     -> throw VectorDecodeException("unknown btc msg scheme: $s")
}

fun decodeSignableMessage(o: JSONObject): SignableMessage = when (val kind = o.getString("kind")) {
    "evmPersonalSign" -> SignableMessage.EvmPersonalSign(message = o.getString("message"))
    "evmTypedDataV4"  -> SignableMessage.EvmTypedDataV4(json = o.getString("json"))
    "bitcoin"         -> SignableMessage.Bitcoin(
        message = o.getString("message"),
        address = o.getString("address"),
        scheme = decodeBtcMsgScheme(o.getString("scheme"))
    )
    "solana"          -> SignableMessage.Solana(messageBase64 = o.getString("message_base64"))
    else              -> throw VectorDecodeException("unknown message kind: $kind")
}
