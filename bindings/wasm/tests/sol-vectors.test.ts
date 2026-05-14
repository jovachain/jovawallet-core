// Phase 6 Solana vector parity tests for the WASM binding.
//
// Mirrors `bindings/kotlin/.../SolVectorsTest.kt`.  Filters on `chain.kind`
// == "solana" / `unsigned_tx.kind` == "solana" / `message.kind` == "solana"
// for the positive-path tests, and `id.startsWith("sol.")` for the error
// vectors.

import { describe, it, expect } from 'vitest';

import { JovaWallet, JovaException } from '../src/index.js';
import vectors from '../../../spec/test-vectors.json';
import { decodeUnsigned, decodeSignableMessage } from './decoders.js';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const all: any[] = (vectors as any).vectors;

describe('SOL address vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'address') continue;
        if (v.input.chain.kind !== 'solana') continue;

        it(`address: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const got = wallet.address({ kind: 'solana' }, 0);
                expect(got.value).toBe(v.expected.address);
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one SOL address vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('SOL sign_tx vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'sign_tx') continue;
        if (v.input.unsigned_tx.kind !== 'solana') continue;

        it(`sign_tx: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const unsigned = decodeUnsigned(v.input.unsigned_tx);
                const signed = wallet.signTx(unsigned);
                expect(signed.raw_hex).toBe(v.expected.signed_hex);
                expect(signed.tx_hash).toBe(v.expected.tx_hash);
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one SOL sign_tx vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('SOL sign_message vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'sign_message') continue;
        if (v.input.message.kind !== 'solana') continue;

        it(`sign_message: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const msg = decodeSignableMessage(v.input.message);
                const sig = wallet.signMessage(msg);
                // SOL convention reuses the `signature_hex` field for the
                // base58-encoded sig (Phase 1 cross-phase convention).
                expect(sig.hex).toBe(v.expected.signature_hex);
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one SOL sign_message vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});

describe('SOL error vectors', () => {
    let ran = 0;
    for (const v of all) {
        if (v.kind !== 'error') continue;
        if (!String(v.id).startsWith('sol.')) continue;

        it(`error: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                let caught: unknown = null;
                try {
                    if (v.input.unsigned_tx) {
                        wallet.signTx(decodeUnsigned(v.input.unsigned_tx));
                    } else if (v.input.message) {
                        wallet.signMessage(decodeSignableMessage(v.input.message));
                    } else {
                        throw new Error(
                            `SOL error vector ${v.id} has neither unsigned_tx nor message`,
                        );
                    }
                } catch (e) {
                    caught = e;
                }
                expect(caught).toBeInstanceOf(JovaException);
                const expected = v.expected.error_variant as string;
                const expectedKind = expected.charAt(0).toLowerCase() + expected.slice(1);
                const payload = (caught as JovaException).error;
                expect(payload.kind).toBe(expectedKind);
                // For SOL errors, reason field is asserted (matches Kotlin
                // SolVectorsTest expectations).
                if ('reason' in payload && v.expected.reason) {
                    expect(payload.reason).toContain(v.expected.reason);
                }
            } finally {
                wallet.destroy();
            }
            ran++;
        });
    }

    it('exercised at least one SOL error vector', () => {
        expect(ran).toBeGreaterThan(0);
    });
});
