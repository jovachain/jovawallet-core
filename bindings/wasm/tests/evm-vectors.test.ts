// Phase 6 EVM vector parity tests for the WASM binding.
//
// Mirrors `bindings/kotlin/.../EvmVectorsTest.kt`: every vector in
// `spec/test-vectors.json` whose chain (for address vectors) or unsigned_tx
// (for sign_tx) is in the EVM family is exercised through the WASM
// `JovaWallet`, asserting byte-identical output against the captured cast
// reference values.

import { describe, it, expect } from 'vitest';

import { JovaWallet, JovaException } from '../src/index.js';
import vectors from '../../../spec/test-vectors.json';
import { decodeChain, decodeEvmUnsigned, decodeSignableMessage } from './decoders.js';

const EVM_KINDS = new Set([
    'ethereum',
    'polygon',
    'bsc',
    'arbitrum',
    'optimism',
    'base',
    'customEvm',
]);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const all: any[] = (vectors as any).vectors;

describe('EVM address vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'address') continue;
        if (!EVM_KINDS.has(v.input.chain.kind)) continue;

        it(`address: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const got = wallet.address(decodeChain(v.input.chain), 0);
                expect(got.value.toLowerCase()).toBe(
                    (v.expected.address as string).toLowerCase(),
                );
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one EVM address vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('EVM sign_tx vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'sign_tx') continue;
        if (v.input.unsigned_tx.kind !== 'evm') continue;

        it(`sign_tx: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const unsigned = { kind: 'evm' as const, ...decodeEvmUnsigned(v.input.unsigned_tx) };
                const signed = wallet.signTx(unsigned);
                expect(signed.raw_hex.toLowerCase()).toBe(
                    (v.expected.signed_hex as string).toLowerCase(),
                );
                expect(signed.tx_hash.toLowerCase()).toBe(
                    (v.expected.tx_hash as string).toLowerCase(),
                );
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one EVM sign_tx vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('EVM sign_message vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'sign_message') continue;
        const k = v.input.message.kind;
        if (k !== 'evmPersonalSign' && k !== 'evmTypedDataV4') continue;

        it(`sign_message: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const msg = decodeSignableMessage(v.input.message);
                const sig = wallet.signMessage(msg);
                expect(sig.hex.toLowerCase()).toBe(
                    (v.expected.signature as string).toLowerCase(),
                );
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one EVM sign_message vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('EVM error vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'error') continue;
        // Phase 1 error vectors go through sign_tx with a malformed tx.
        if (!v.input.unsigned_tx) continue;
        if (v.input.unsigned_tx.kind !== 'evm') continue;
        // ID convention: Phase 1 used `phase1.error.*`; Phase 6 keeps that
        // filter to avoid pulling in btc.error / sol.error / xrp.error
        // vectors which are scoped to their per-chain test files.
        if (!String(v.id).startsWith('phase1.error.')) continue;

        it(`error: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const unsigned = { kind: 'evm' as const, ...decodeEvmUnsigned(v.input.unsigned_tx) };
                let caught: unknown = null;
                try {
                    wallet.signTx(unsigned);
                } catch (e) {
                    caught = e;
                }
                expect(caught).toBeInstanceOf(JovaException);
                // The Rust JovaError variant names map onto the WASM
                // JovaErrorPayload `kind` discriminator with camelCase
                // ("MalformedUnsignedTx" → "malformedUnsignedTx").
                const expected = (v.expected.error_variant as string);
                const expectedKind = expected.charAt(0).toLowerCase() + expected.slice(1);
                const payload = (caught as JovaException).error;
                expect(payload.kind).toBe(expectedKind);
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one EVM error vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});
