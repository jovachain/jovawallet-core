package io.jova.core

import uniffi.jova_core_ffi.EvmUnsigned
import uniffi.jova_core_ffi.AccessListItem

object JovaCoreVersion {
    const val VALUE = "0.1.0"
}

// Helper for the most common case: build an EVM transfer without filling in the access list.
fun evmTransfer(
    chainId: ULong,
    nonce: ULong,
    to: String,
    valueWei: String,
    gasLimit: ULong = 21_000UL,
    maxFeePerGas: String,
    maxPriorityFeePerGas: String
): EvmUnsigned = EvmUnsigned(
    chainId = chainId,
    nonce = nonce,
    to = to,
    value = valueWei,
    gasLimit = gasLimit,
    maxFeePerGas = maxFeePerGas,
    maxPriorityFeePerGas = maxPriorityFeePerGas,
    `data` = "0x",
    accessList = emptyList()
)
