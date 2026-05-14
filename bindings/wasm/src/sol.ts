// `@jovachain/wallet-core/sol` — Solana-only convenience entrypoint.

import { JovaWallet, init, isValidAddress } from './index.js';
import type {
    Mnemonic,
    JovaChain,
    UnsignedTx,
    SignableMessage,
    SignedTx,
    Signature,
    Address,
    JovaErrorPayload,
} from './types.js';
import { JovaException } from './types.js';

export { init, JovaWallet, JovaException };
export type {
    Mnemonic,
    UnsignedTx,
    SignableMessage,
    SignedTx,
    Signature,
    Address,
    JovaErrorPayload,
};

export const SOLANA: JovaChain = { kind: 'solana' };

/**
 * Validate a base58-encoded Solana address (32-byte Ed25519 public key).
 */
export function isValidSolAddress(addr: string): boolean {
    return isValidAddress(addr, SOLANA);
}

/**
 * Wrap a base64-encoded `VersionedMessage` and its recent blockhash in the
 * `UnsignedTx::Solana` envelope.  The signer enforces that the embedded
 * blockhash equals `recentBlockhash` byte-for-byte; pass the same value you
 * fetched from the RPC.
 */
export function solanaTx(opts: {
    messageBase64: string;
    recentBlockhash: string;
}): UnsignedTx {
    return {
        kind: 'solana',
        message_base64: opts.messageBase64,
        recent_blockhash: opts.recentBlockhash,
    };
}

/**
 * Build a `SignableMessage::Solana` from a base64-encoded byte payload.  No
 * canonical Solana message envelope exists; Phantom, Solflare and Backpack
 * all sign the raw payload with the leaf ed25519 key.
 */
export function solanaMessage(messageBase64: string): SignableMessage {
    return { kind: 'solana', message_base64: messageBase64 };
}
