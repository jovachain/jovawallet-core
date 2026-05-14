// Vector JSON → WASM-input shape decoders.
//
// Mirrors `bindings/kotlin/.../VectorDecoders.kt`.  Each decoder converts a
// raw entry from `spec/test-vectors.json` into the camelCase / snake_case
// shape that serde-wasm-bindgen expects on the WASM side.
//
// The spec JSON already uses camelCase for `EvmUnsigned` fields (chainId,
// nonce, ...) and snake_case for inner `AccessListItem` fields (storage_keys)
// because that is how the Rust core's serde-tagged enums serialize.  This
// file therefore mostly performs `BigInt(...)` widening for u64 fields and
// re-shapes from JSON-style objects to TS objects.

import type {
    JovaChain,
    UnsignedTx,
    EvmUnsigned,
    SignableMessage,
    AccessListItem,
} from '../src/types.js';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function decodeChain(o: any): JovaChain {
    switch (o.kind) {
        case 'ethereum':
        case 'polygon':
        case 'bsc':
        case 'arbitrum':
        case 'optimism':
        case 'base':
        case 'bitcoin':
        case 'solana':
        case 'xrp':
            return { kind: o.kind };
        case 'customEvm':
            return { kind: 'customEvm', chainId: BigInt(o.chainId) };
        default:
            throw new Error(`unknown chain kind: ${o.kind}`);
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function decodeEvmUnsigned(o: any): EvmUnsigned {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const accessList: AccessListItem[] = ((o.accessList ?? []) as any[]).map((i) => ({
        address: i.address,
        storage_keys: i.storage_keys ?? i.storageKeys ?? [],
    }));
    return {
        chainId: BigInt(o.chainId),
        nonce: BigInt(o.nonce),
        to: o.to,
        value: o.value,
        gasLimit: BigInt(o.gasLimit),
        maxFeePerGas: o.maxFeePerGas,
        maxPriorityFeePerGas: o.maxPriorityFeePerGas,
        data: o.data,
        accessList,
    };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function decodeUnsigned(o: any): UnsignedTx {
    switch (o.kind) {
        case 'evm':
            return { kind: 'evm', ...decodeEvmUnsigned(o) };
        case 'bitcoin':
            return { kind: 'bitcoin', psbt_base64: o.psbt_base64 };
        case 'solana':
            return {
                kind: 'solana',
                message_base64: o.message_base64,
                recent_blockhash: o.recent_blockhash,
            };
        case 'xrp':
            return { kind: 'xrp', tx_json: o.tx_json };
        default:
            throw new Error(`unknown unsigned kind: ${o.kind}`);
    }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function decodeSignableMessage(o: any): SignableMessage {
    switch (o.kind) {
        case 'evmPersonalSign':
            return { kind: 'evmPersonalSign', message: o.message };
        case 'evmTypedDataV4':
            return { kind: 'evmTypedDataV4', json: o.json };
        case 'solana':
            return { kind: 'solana', message_base64: o.message_base64 };
        case 'bitcoin':
            return {
                kind: 'bitcoin',
                message: o.message,
                address: o.address,
                scheme: o.scheme,
            };
        default:
            throw new Error(`unknown message kind: ${o.kind}`);
    }
}
