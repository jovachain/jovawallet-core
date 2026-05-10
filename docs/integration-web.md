# Integration: Web (WASM)

How a browser-based or Node-based wallet consumes `jovawallet-core`. This guide covers Phase 6 of the plan; the binding ships at v1.1.0.

> **Honest disclaimer on chain coverage.** The WASM build target has compiled continuously from Phase 0 onward — every PR's CI proves that `jova-core-wasm` builds for `wasm32-unknown-unknown`. But functional vector tests on WASM only land at v1.1.0, and a chain crate may need feature-flagging to compile. The Phase -1 feasibility spike documented which chains compile cleanly to WASM in 2026. If a chain's underlying Rust crate fights WASM (heavy `tokio` features, `std::os` dependencies, etc.), that chain may ship to WASM later than to native bindings. The npm package's `README.md` documents which chains are fully functional vs. native-only at the point of any given release. The architecture itself supports every chain on WASM; production-quality coverage is gated on each crate's WASM friendliness.

The web binding is an npm package: `@jovachain/wallet-core`. It contains a Rust core compiled to WebAssembly via `wasm-bindgen`, plus TypeScript type definitions.

## Adding the dependency

```bash
npm install @jovachain/wallet-core
# or
pnpm add @jovachain/wallet-core
# or
yarn add @jovachain/wallet-core
```

The package ships ESM and CJS variants plus TypeScript types. It's tree-shakeable; bundlers strip unused chains from the WASM output.

## Browser usage

```typescript
import { JovaWallet, JovaChain, Strength } from '@jovachain/wallet-core';
import init from '@jovachain/wallet-core/init';

await init();   // loads the WASM module; required once per page

const mnemonic = JovaWallet.createMnemonic(Strength.bits128);
console.log('words:', mnemonic.words);

const wallet = JovaWallet.fromMnemonic(mnemonic);
try {
    const eth = wallet.address(JovaChain.Ethereum);
    const btc = wallet.address(JovaChain.Bitcoin);
    console.log('ETH:', eth.value);
    console.log('BTC:', btc.value);
} finally {
    wallet.destroy();   // CRITICAL: zeroizes WASM memory holding the seed
}
```

## Node usage

```typescript
import { JovaWallet, JovaChain, Strength } from '@jovachain/wallet-core';
// Node target auto-initializes; no explicit init() call needed.

const mnemonic = JovaWallet.createMnemonic(Strength.bits256);
const wallet = JovaWallet.fromMnemonic(mnemonic);
try {
    const eth = wallet.address(JovaChain.Ethereum);
    console.log('ETH:', eth.value);
} finally {
    wallet.destroy();
}
```

---

## Bundle size

A signing-only WASM binary lands around 600 kB–1.5 MB gzipped depending on which chains are bundled. Tree-shaking strips unused chains.

If your app only needs Ethereum:

```typescript
import { JovaWalletEvmOnly } from '@jovachain/wallet-core/evm';
```

Per-chain entrypoints land in v1.2+. The v1.1 release ships the full bundle only.

## Web Workers

Crypto signing should run in a Web Worker — keeps the main thread responsive and isolates secret material from page-level scripts.

```typescript
// worker.ts
import { JovaWallet, JovaChain } from '@jovachain/wallet-core';
import init from '@jovachain/wallet-core/init';

let wallet: JovaWallet | null = null;

self.addEventListener('message', async (e) => {
    if (e.data.type === 'init') {
        await init();
        wallet = JovaWallet.fromMnemonic(e.data.mnemonic);
        self.postMessage({ type: 'ready' });
    }

    if (e.data.type === 'sign') {
        const signed = wallet!.sign(e.data.tx);
        self.postMessage({ type: 'signed', signed });
    }

    if (e.data.type === 'destroy') {
        wallet?.destroy();
        wallet = null;
        self.postMessage({ type: 'destroyed' });
        self.close();
    }
});
```

```typescript
// main.ts
const worker = new Worker('/wallet-worker.js', { type: 'module' });
worker.addEventListener('message', (e) => { /* … */ });
worker.postMessage({ type: 'init', mnemonic });
worker.postMessage({ type: 'sign', tx });
worker.postMessage({ type: 'destroy' });
```

This pattern:

- Keeps the seed in worker memory, separate from page memory.
- Prevents a page-level XSS from immediately accessing the seed (defense in depth).
- Survives common ad scripts and analytics that run on the main thread.

---

## Memory and clearing

**You must call `wallet.destroy()`.** The JavaScript GC does not run finalizers synchronously enough for crypto-grade clearing. Without `destroy()`, the seed bytes sit in WASM linear memory until the page unloads or the GC happens to collect — which may be never.

Pattern:

```typescript
let wallet: JovaWallet | null = null;
try {
    wallet = JovaWallet.fromMnemonic(mnemonic);
    return wallet.signTx(tx);
} finally {
    wallet?.destroy();
}
```

Or with an `AsyncDisposable` if you target ES2026+:

```typescript
{
    using wallet = JovaWallet.fromMnemonic(mnemonic);
    return wallet.signTx(tx);
}   // wallet[Symbol.dispose]() runs automatically
```

We implement `Symbol.dispose` on `JovaWallet` for this pattern.

---

## Storage

The browser has no Keychain. Options:

- **`indexedDB` with subtle-crypto AES-GCM**: app derives a key via WebAuthn PRF or user passphrase, encrypts the mnemonic, stores in IndexedDB.
- **WebAuthn PRF**: an authenticator (TouchID, YubiKey) provides a stable secret keyed to user verification. Best UX in 2026.
- **OS-bridged stores**: native helper apps (e.g., a Tauri shell) can use the OS keychain on the user's behalf.

The SDK does not handle storage. App layer.

A reference implementation:

```typescript
async function storeMnemonic(words: string, prfKey: ArrayBuffer): Promise<void> {
    const enc = new TextEncoder().encode(words);
    const key = await crypto.subtle.importKey(
        'raw', prfKey, 'AES-GCM', false, ['encrypt', 'decrypt']
    );
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, enc);

    const db = await openDb();
    await db.put('mnemonic', { iv, ct });
}

async function loadMnemonic(prfKey: ArrayBuffer): Promise<Uint8Array> {
    const db = await openDb();
    const { iv, ct } = await db.get('mnemonic');
    const key = await crypto.subtle.importKey(
        'raw', prfKey, 'AES-GCM', false, ['encrypt', 'decrypt']
    );
    const pt = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, ct);
    return new Uint8Array(pt);
}
```

Pass the resulting `Uint8Array` to `JovaWallet.fromMnemonicBuffer(...)`.

---

## React example

```typescript
import { useEffect, useState } from 'react';
import { JovaWallet, JovaChain, Address } from '@jovachain/wallet-core';
import init from '@jovachain/wallet-core/init';

export function useJovaAddresses(mnemonic: string, chains: JovaChain[]): Address[] | null {
    const [addresses, setAddresses] = useState<Address[] | null>(null);

    useEffect(() => {
        let cancelled = false;
        (async () => {
            await init();
            const wallet = JovaWallet.fromMnemonic({ words: mnemonic, passphrase: '' });
            try {
                const result = chains.map(c => wallet.address(c));
                if (!cancelled) setAddresses(result);
            } finally {
                wallet.destroy();
            }
        })();
        return () => { cancelled = true; };
    }, [mnemonic, chains]);

    return addresses;
}
```

In production, move this into a Web Worker (see above).

---

## Wagmi / viem integration

The web binding does not implement the Wagmi/viem signer interfaces directly — that's a separate adapter package on the roadmap (`@jovachain/wallet-core-viem`). Until then:

```typescript
import { createWalletClient, custom } from 'viem';
import { JovaWallet, UnsignedTx } from '@jovachain/wallet-core';

const wallet = JovaWallet.fromMnemonic({ words, passphrase: '' });

const client = createWalletClient({
    chain: mainnet,
    transport: custom({
        async request({ method, params }) {
            switch (method) {
                case 'eth_signTransaction': {
                    const tx: UnsignedTx = {
                        kind: 'evm',
                        chainId: 1n,
                        nonce: BigInt(params[0].nonce),
                        to: params[0].to,
                        value: params[0].value,
                        gasLimit: BigInt(params[0].gas),
                        maxFeePerGas: params[0].maxFeePerGas,
                        maxPriorityFeePerGas: params[0].maxPriorityFeePerGas,
                        data: params[0].data ?? '0x',
                    };
                    return wallet.signTx(tx).rawHex;
                }
                case 'personal_sign': {
                    return wallet.signMessage({
                        kind: 'evmPersonalSign',
                        message: params[0],
                    }).hex;
                }
                default:
                    throw new Error(`unsupported: ${method}`);
            }
        },
    }),
});
```

---

## TypeScript types

The package ships `.d.ts` files generated by `wasm-bindgen`'s TypeScript output plus the hand-written `index.d.ts` re-export layer. Highlights:

```typescript
export interface Mnemonic {
    words: string;
    passphrase: string;
}

export type JovaChain =
    | 'ethereum' | 'polygon' | 'bsc'
    | 'arbitrum' | 'optimism' | 'base'
    | 'bitcoin' | 'solana' | 'xrp'
    | { kind: 'customEvm'; chainId: bigint };

export type UnsignedTx =
    | { kind: 'evm'; chainId: bigint; nonce: bigint; to: string; value: string;
        gasLimit: bigint; maxFeePerGas: string; maxPriorityFeePerGas: string;
        data: string; accessList?: AccessList }
    | { kind: 'bitcoin'; psbtBase64: string }
    | { kind: 'solana'; messageBase64: string; recentBlockhash: string }
    | { kind: 'xrp'; txJson: string };

export class JovaWallet implements Disposable {
    static createMnemonic(strength: Strength): Mnemonic;
    static isValidMnemonic(words: string, passphrase?: string): boolean;
    static isValidAddress(addr: string, chain: JovaChain): boolean;
    static fromMnemonic(mnemonic: Mnemonic): JovaWallet;
    static fromMnemonicBuffer(buf: MnemonicBuffer): JovaWallet;

    address(chain: JovaChain, account?: number): Address;
    signTx(unsigned: UnsignedTx): SignedTx;
    signMessage(msg: SignableMessage): Signature;

    destroy(): void;
    [Symbol.dispose](): void;
}
```

Discriminated unions on `kind` enable exhaustive `switch` typed-checks.

---

## CSP and SRI

For browsers with strict Content Security Policy:

```
Content-Security-Policy: script-src 'self' 'wasm-unsafe-eval'; ...
```

`wasm-unsafe-eval` is required. (It is the appropriate token for executing WASM; `unsafe-eval` is broader and not needed.)

Subresource Integrity for the WASM file: the npm package includes `wasm-sha384.json` with the integrity hash; bundlers should reference it.

---

## Don'ts

- Don't forget `wallet.destroy()`.
- Don't keep a `JovaWallet` reference in long-lived state (Redux, Zustand, etc.).
- Don't transfer a `JovaWallet` instance to a Web Worker via `postMessage` — it isn't transferable. Construct in the worker; transfer the mnemonic bytes.
- Don't hold the `mnemonic.words` string longer than necessary; JS strings are immutable and not zeroizable.
- Don't run signing on the main thread in production.
- Don't persist `signed.rawHex` to localStorage / IndexedDB — it's PII.

---

## Sample app

`examples/web-sample/` is a Vite + TypeScript app demonstrating the full flow with a Web Worker. Use it as a copy-and-modify reference.
