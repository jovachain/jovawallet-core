// `@jovachain/wallet-core/evm` — EVM-only convenience entrypoint.
//
// Re-exports the core wallet plus EVM chain helpers.  Bundlers tree-shake any
// unused chain-specific helpers, so a browser app that only signs EVM txs can
// import from this subpath and skip the SOL helpers (and vice versa).

import { JovaWallet, init, isValidAddress } from './index.js';
import type {
    Mnemonic,
    JovaChain,
    EvmUnsigned,
    UnsignedTx,
    SignableMessage,
    SignedTx,
    Signature,
    Address,
    AccessListItem,
    JovaErrorPayload,
} from './types.js';
import { JovaException } from './types.js';

export { init, JovaWallet, JovaException };
export type {
    Mnemonic,
    EvmUnsigned,
    UnsignedTx,
    SignableMessage,
    SignedTx,
    Signature,
    Address,
    AccessListItem,
    JovaErrorPayload,
};

/**
 * Canonical EVM chain enum values.  `customEvm` is a function so callers can
 * pass an arbitrary chain id at runtime.
 */
export const EVM_CHAINS = {
    ethereum: { kind: 'ethereum' as const },
    polygon: { kind: 'polygon' as const },
    bsc: { kind: 'bsc' as const },
    arbitrum: { kind: 'arbitrum' as const },
    optimism: { kind: 'optimism' as const },
    base: { kind: 'base' as const },
    customEvm: (chainId: bigint): JovaChain => ({ kind: 'customEvm', chainId }),
};

/**
 * Validate a checksummed EVM address (EIP-55).  Chain selection is irrelevant
 * for address validation — the same routine applies to every EVM-family
 * chain.
 */
export function isValidEvmAddress(addr: string): boolean {
    return isValidAddress(addr, EVM_CHAINS.ethereum);
}

/**
 * Build an EIP-1559 transfer (transaction type 2) without an access list.
 *
 * Callers must supply the wei value as a decimal string and gas-price fields
 * as decimal strings (matching the engine's expected wire format).  Use
 * `data: "0x"` and `accessList: []` for pure-ETH transfers; this helper
 * fills both for you.
 */
export function evmTransfer(opts: {
    chainId: bigint;
    nonce: bigint;
    to: string;
    valueWei: string;
    gasLimit?: bigint;
    maxFeePerGas: string;
    maxPriorityFeePerGas: string;
}): UnsignedTx {
    return {
        kind: 'evm',
        chainId: opts.chainId,
        nonce: opts.nonce,
        to: opts.to,
        value: opts.valueWei,
        gasLimit: opts.gasLimit ?? 21_000n,
        maxFeePerGas: opts.maxFeePerGas,
        maxPriorityFeePerGas: opts.maxPriorityFeePerGas,
        data: '0x',
        accessList: [],
    };
}
