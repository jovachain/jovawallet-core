// Hand-written TypeScript types mirroring the serde-shaped JS values that
// `jova-core-wasm` exchanges with the host.  Discriminated unions are used so
// the TS compiler can narrow on `kind`.
//
// Field names follow the camelCase convention applied by `serde(rename_all =
// "camelCase")` in the Rust crates.

// ---------- Mnemonic ----------

export interface Mnemonic {
    words: string;
    passphrase: string;
}

// ---------- Chain enum ----------

export type JovaChain =
    | { kind: 'ethereum' }
    | { kind: 'polygon' }
    | { kind: 'bsc' }
    | { kind: 'arbitrum' }
    | { kind: 'optimism' }
    | { kind: 'base' }
    | { kind: 'bitcoin' }
    | { kind: 'solana' }
    | { kind: 'xrp' }
    | { kind: 'customEvm'; chainId: bigint };

// ---------- UnsignedTx variants ----------

// NOTE: `storage_keys` is snake_case because Rust `AccessListItem` does not
// carry a `#[serde(rename_all = "camelCase")]` attribute — only the parent
// `EvmUnsigned` struct does.  Keep these names aligned with the Rust struct.
export interface AccessListItem {
    address: string;
    storage_keys: string[];
}

export interface EvmUnsigned {
    chainId: bigint;
    nonce: bigint;
    to: string;
    value: string; // wei decimal string
    gasLimit: bigint;
    maxFeePerGas: string;
    maxPriorityFeePerGas: string;
    data: string; // 0x-prefixed hex
    accessList: AccessListItem[];
}

// The Rust `UnsignedTx::Evm(EvmUnsigned)` variant is encoded as
// `{ kind: "evm", ...EvmUnsigned }` because serde flattens the inner struct's
// fields up under the enum-tag object (single tuple field of a tagged enum).
export type UnsignedTx =
    | ({ kind: 'evm' } & EvmUnsigned)
    | { kind: 'bitcoin'; psbt_base64: string }
    | { kind: 'solana'; message_base64: string; recent_blockhash: string }
    | { kind: 'xrp'; tx_json: string };

// ---------- SignableMessage variants ----------

export type BtcMsgScheme = 'bip322' | 'legacy';

export type SignableMessage =
    | { kind: 'evmPersonalSign'; message: string }
    | { kind: 'evmTypedDataV4'; json: string }
    | { kind: 'solana'; message_base64: string }
    | { kind: 'bitcoin'; message: string; address: string; scheme: BtcMsgScheme };

// ---------- Outputs ----------

// `chain` here is the canonical short label string emitted by jova-core-chains
// (e.g. "ethereum", "polygon", "solana").  This intentionally mirrors the
// Rust `Address.chain: String` shape rather than echoing the full JovaChain
// object — keeps native + wasm outputs byte-identical.
export interface Address {
    chain: string;
    value: string;
}

export interface SignedTx {
    chain: string;
    raw_hex: string;
    tx_hash: string;
}

export interface Signature {
    hex: string;
}

// ---------- Error payloads ----------

export type JovaErrorPayload =
    | { kind: 'invalidMnemonic' }
    | { kind: 'invalidPassphrase' }
    | { kind: 'invalidAddress'; chain: string }
    | { kind: 'unsupportedChain'; chain: string }
    | { kind: 'malformedUnsignedTx'; reason: string }
    | { kind: 'malformedSignableMessage'; reason: string }
    | { kind: 'signingFailed'; reason: string }
    | { kind: 'internal'; reason: string };

export class JovaException extends Error {
    constructor(public readonly error: JovaErrorPayload) {
        super(JSON.stringify(error));
        this.name = 'JovaException';
    }
}
