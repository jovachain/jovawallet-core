// Phase 6: assert BTC and XRP variants are rejected at the WASM boundary
// with a clean `unsupportedChain` payload.  Per the 2026-05-11 user decision,
// these chains do not have browser signing support in v1.1.  See
// `bindings/wasm/COVERAGE.md`.

import { describe, it, expect } from 'vitest';

import { JovaWallet, JovaException } from '../src/index.js';

const MNEMONIC = {
    words:
        'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about',
    passphrase: '',
};

describe('WASM unsupported-chain gating', () => {
    it('address(bitcoin) returns unsupportedChain', () => {
        const wallet = JovaWallet.fromMnemonic(MNEMONIC);
        try {
            expect(() => wallet.address({ kind: 'bitcoin' }, 0)).toThrow(JovaException);
            try {
                wallet.address({ kind: 'bitcoin' }, 0);
            } catch (e) {
                const payload = (e as JovaException).error;
                expect(payload.kind).toBe('unsupportedChain');
                if (payload.kind === 'unsupportedChain') {
                    expect(payload.chain).toBe('bitcoin');
                }
            }
        } finally {
            wallet.destroy();
        }
    });

    it('address(xrp) returns unsupportedChain', () => {
        const wallet = JovaWallet.fromMnemonic(MNEMONIC);
        try {
            try {
                wallet.address({ kind: 'xrp' }, 0);
                throw new Error('expected throw');
            } catch (e) {
                expect(e).toBeInstanceOf(JovaException);
                const payload = (e as JovaException).error;
                expect(payload.kind).toBe('unsupportedChain');
                if (payload.kind === 'unsupportedChain') {
                    expect(payload.chain).toBe('xrp');
                }
            }
        } finally {
            wallet.destroy();
        }
    });

    it('signTx({ kind: "bitcoin" }) returns unsupportedChain', () => {
        const wallet = JovaWallet.fromMnemonic(MNEMONIC);
        try {
            try {
                wallet.signTx({ kind: 'bitcoin', psbt_base64: 'AAAA' });
                throw new Error('expected throw');
            } catch (e) {
                expect(e).toBeInstanceOf(JovaException);
                const payload = (e as JovaException).error;
                expect(payload.kind).toBe('unsupportedChain');
                if (payload.kind === 'unsupportedChain') {
                    expect(payload.chain).toBe('bitcoin');
                }
            }
        } finally {
            wallet.destroy();
        }
    });

    it('signTx({ kind: "xrp" }) returns unsupportedChain', () => {
        const wallet = JovaWallet.fromMnemonic(MNEMONIC);
        try {
            try {
                wallet.signTx({ kind: 'xrp', tx_json: '{}' });
                throw new Error('expected throw');
            } catch (e) {
                expect(e).toBeInstanceOf(JovaException);
                const payload = (e as JovaException).error;
                expect(payload.kind).toBe('unsupportedChain');
                if (payload.kind === 'unsupportedChain') {
                    expect(payload.chain).toBe('xrp');
                }
            }
        } finally {
            wallet.destroy();
        }
    });

    it('signMessage({ kind: "bitcoin" }) returns unsupportedChain', () => {
        const wallet = JovaWallet.fromMnemonic(MNEMONIC);
        try {
            try {
                wallet.signMessage({
                    kind: 'bitcoin',
                    message: 'hi',
                    address: 'bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu',
                    scheme: 'bip322',
                });
                throw new Error('expected throw');
            } catch (e) {
                expect(e).toBeInstanceOf(JovaException);
                const payload = (e as JovaException).error;
                expect(payload.kind).toBe('unsupportedChain');
                if (payload.kind === 'unsupportedChain') {
                    expect(payload.chain).toBe('bitcoin');
                }
            }
        } finally {
            wallet.destroy();
        }
    });
});
