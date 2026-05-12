import { describe, it, expect } from 'vitest';
import { createRequire } from 'module';
import vectors from '../../../spec/test-vectors.json';

// wasm-pack --target nodejs generates CJS (exports.*, require, __dirname).
// Use createRequire to import it from an ESM context.
const require = createRequire(import.meta.url);
const { isValidMnemonic } = require('../pkg/jova_core_wasm.js') as {
    isValidMnemonic: (words: string, passphrase: string) => boolean;
};

describe('Phase 0 hello-world', () => {
    it('rejects the negative-mnemonic vector', () => {
        const v = vectors.vectors.find(
            (x: any) => x.id === 'phase0.mnemonic_validation_neg.gibberish'
        );
        expect(v).toBeDefined();
        const words = v!.input.words as string;
        const passphrase = (v!.input.passphrase as string | undefined) ?? '';
        const expected = (v!.expected as any).valid as boolean;

        expect(isValidMnemonic(words, passphrase)).toBe(expected);
    });
});
