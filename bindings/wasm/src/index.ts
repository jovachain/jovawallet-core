// Public TypeScript entry point for @jovachain/wallet-core.
//
// wasm-pack --target nodejs emits CommonJS that synchronously loads the
// .wasm sidecar at require() time.  The CJS module is consumed through
// createRequire so this file can stay ESM (matches the package "type":
// "module" declaration).  No async init() is needed.

import { createRequire } from 'module';

import type {
    Mnemonic,
    JovaChain,
    UnsignedTx,
    SignableMessage,
    Address,
    SignedTx,
    Signature,
    EvmUnsigned,
    AccessListItem,
    BtcMsgScheme,
    JovaErrorPayload,
} from './types.js';
import { JovaException } from './types.js';

export type {
    Mnemonic,
    JovaChain,
    UnsignedTx,
    SignableMessage,
    Address,
    SignedTx,
    Signature,
    EvmUnsigned,
    AccessListItem,
    BtcMsgScheme,
    JovaErrorPayload,
};
export { JovaException };

// ---------- Load the wasm-pack CJS shim ----------

interface RawModule {
    JovaWallet: new (mnemonic: Mnemonic) => RawWallet;
    createMnemonic: (bits256: boolean) => Mnemonic;
    isValidMnemonic: (words: string, passphrase: string) => boolean;
    isValidAddress: (addr: string, chain: JovaChain) => boolean;
}

interface RawWallet {
    address(chain: JovaChain, account: number): Address;
    signTx(unsigned: UnsignedTx, account: number): SignedTx;
    signMessage(msg: SignableMessage, account: number): Signature;
    destroy(): void;
    free(): void;
}

const require = createRequire(import.meta.url);
// eslint-disable-next-line @typescript-eslint/no-var-requires
const raw = require('../pkg/jova_core_wasm.js') as RawModule;

// ---------- Free functions ----------

/**
 * Generate a new BIP-39 mnemonic.  `bits256` = false produces 12 words,
 * `bits256` = true produces 24 words.  Default: 12 words.
 *
 * The returned object holds a plain string; for crypto-grade lifetime control
 * use `JovaWallet.fromMnemonic` + `destroy()` (or `using`) and discard the
 * mnemonic from JS state once the wallet is constructed.
 */
export function createMnemonic(bits256: boolean = false): Mnemonic {
    return raw.createMnemonic(bits256);
}

/**
 * BIP-39 mnemonic validation (English wordlist + checksum).
 */
export function isValidMnemonic(words: string, passphrase: string = ''): boolean {
    return raw.isValidMnemonic(words, passphrase);
}

/**
 * Per-chain address validation.  Returns `false` for malformed input rather
 * than throwing — call sites typically guard on this before constructing a
 * tx.
 */
export function isValidAddress(addr: string, chain: JovaChain): boolean {
    try {
        return raw.isValidAddress(addr, chain);
    } catch (e) {
        throw asJovaException(e);
    }
}

/**
 * `init()` is a compatibility shim for callers that expect the wasm-pack
 * `--target web` pattern (an async initializer).  In the `--target nodejs`
 * build the WASM is loaded synchronously at module-load time, so this is a
 * no-op that resolves immediately.  Keeping the export means consumer code
 * written for either target works in both.
 */
export async function init(): Promise<void> {
    return;
}

/**
 * A Jova signing wallet.  Implements `Disposable` (TypeScript 5.5+) so
 * `using wallet = JovaWallet.fromMnemonic(...)` automatically destroys the
 * underlying WASM-side seed buffer when the scope exits.
 *
 * Without `using`, you MUST call `wallet.destroy()` manually — the JS GC
 * does not run finalizers synchronously enough for crypto-grade clearing.
 */
export class JovaWallet implements Disposable {
    private constructor(private wasm: RawWallet) {}

    static fromMnemonic(mnemonic: Mnemonic): JovaWallet {
        try {
            return new JovaWallet(new raw.JovaWallet(mnemonic));
        } catch (e) {
            throw asJovaException(e);
        }
    }

    address(chain: JovaChain, account: number = 0): Address {
        try {
            return this.wasm.address(chain, account);
        } catch (e) {
            throw asJovaException(e);
        }
    }

    signTx(unsigned: UnsignedTx, account: number = 0): SignedTx {
        try {
            return this.wasm.signTx(unsigned, account);
        } catch (e) {
            throw asJovaException(e);
        }
    }

    signMessage(msg: SignableMessage, account: number = 0): Signature {
        try {
            return this.wasm.signMessage(msg, account);
        } catch (e) {
            throw asJovaException(e);
        }
    }

    destroy(): void {
        this.wasm.destroy();
    }

    [Symbol.dispose](): void {
        this.destroy();
    }
}

// ---------- Error coercion ----------

function asJovaException(e: unknown): JovaException {
    if (typeof e === 'object' && e !== null && 'kind' in (e as object)) {
        return new JovaException(e as JovaErrorPayload);
    }
    return new JovaException({ kind: 'internal', reason: String(e) });
}
