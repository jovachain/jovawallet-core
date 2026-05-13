package io.jova.core

import org.json.JSONObject
import uniffi.jova_core_ffi.AccessListItem
import uniffi.jova_core_ffi.EvmUnsigned
import uniffi.jova_core_ffi.JovaChain
import uniffi.jova_core_ffi.SignableMessage

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

fun decodeSignableMessage(o: JSONObject): SignableMessage = when (val kind = o.getString("kind")) {
    "evmPersonalSign" -> SignableMessage.EvmPersonalSign(message = o.getString("message"))
    "evmTypedDataV4"  -> SignableMessage.EvmTypedDataV4(json = o.getString("json"))
    else              -> throw VectorDecodeException("unknown message kind: $kind")
}
