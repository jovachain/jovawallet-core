# Phase 6: WASM Functional + npm Publish

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `@jovachain/wallet-core` published to npm with full functional vector parity for every WASM-supported chain. Browser + Node consumption documented and demonstrated. Bundle-size optimized. Web Worker + `Symbol.dispose` supported. Tag `v1.1.0`.

**Architecture:** Expand `crates/jova-core-wasm/` to expose the full `JovaWallet` surface via `wasm-bindgen`. Build `bindings/wasm/` as an npm package with TypeScript types. Per-chain entrypoints for tree-shaking. Web Worker example demonstrates the recommended consumption pattern.

**Tech Stack:** Same as Phase 0+, plus: vitest 2.x, esbuild for bundling, TypeScript 5.5+ (Symbol.dispose support), `getrandom` 0.3 with `wasm_js` feature.

**Preconditions:**
- Phase 5 complete; `v1.0.0` tagged.
- Phase -1 feasibility report's WASM column is filled in (which chains compile to WASM).
- Native bindings (Swift, Kotlin) have been stable for at least one full v1.0.x release cycle.
- The `@jovachain` npm scope is registered and the team has publish access.

**Exit criteria:**
- All vectors that apply to WASM-supported chains pass byte-identically against the Rust core via the WASM binding.
- Bundle size: gzip-compressed WASM + JS shim < 2 MB for the full bundle, < 500 kB for an EVM-only entrypoint.
- Web Worker example runs end-to-end in `examples/web-sample/`.
- `using JovaWallet` syntax (`Symbol.dispose`) works in TS 5.5+ environments.
- `@jovachain/wallet-core@1.1.0` published to npm.
- `v1.1.0` tagged on `main`.

---

## Task 1: Audit WASM coverage from feasibility report; feature-flag any uncooperative chain

**Files:**
- Modify: `crates/jova-core-wasm/Cargo.toml`
- Modify: `crates/jova-core-wasm/src/lib.rs`
- Create: `bindings/wasm/COVERAGE.md`

- [ ] **Step 1: Read the feasibility report's WASM column**

```bash
grep -A30 "wasm32-unknown-unknown" docs/feasibility-report.md
```

For each chain:
- ✅ on WASM → enabled in the WASM crate by default.
- ❌ on WASM → feature-flagged off; documented in `bindings/wasm/COVERAGE.md`.
- (partial) → enabled with reduced functionality; documented.

The most likely problem area in 2026 is Solana (depending on whether the Anza split crates are clean WASM targets) and possibly XRP (less ecosystem investment).

- [ ] **Step 2: Update `crates/jova-core-wasm/Cargo.toml` features**

```toml
[features]
default = ["chain-evm", "chain-btc", "chain-sol", "chain-xrp"]
chain-evm = []   # no extra deps; EVM is handled in jova-core unconditionally
chain-btc = []
chain-sol = []
chain-xrp = []
```

If a chain is documented as WASM-incompatible, remove it from `default` and document the gap. The WASM build emits `JovaError::UnsupportedChain` at runtime when an unsupported variant is constructed — apps see a clean error, not a build failure.

- [ ] **Step 3: COVERAGE.md**

`bindings/wasm/COVERAGE.md`:

```markdown
# WASM Chain Coverage

The npm package compiles every chain by default, but some chains may be
feature-flagged off if their underlying Rust crate fights the WASM target.
This file documents the v1.1.0 reality.

| Chain | WASM status | Notes |
|---|---|---|
| Ethereum / Polygon / BSC / Arbitrum / Optimism / Base / customEvm | ✅ Full | alloy is WASM-clean |
| Bitcoin | ✅ / ⚠️ | bdk_wallet status from feasibility report |
| Solana | ✅ / ⚠️ | Anza split crates status from feasibility report |
| XRP | ✅ / ⚠️ | xrpl status from feasibility report |

If any chain shows ⚠️, calling its sign methods returns
`JovaError.UnsupportedChain`. Native bindings (Swift, Kotlin) and the Rust crate
have full coverage; only WASM is constrained.
```

- [ ] **Step 4: Commit**

```bash
git add crates/jova-core-wasm/Cargo.toml bindings/wasm/COVERAGE.md
git commit -m "feat(wasm): chain feature flags + honest coverage doc"
```

---

## Task 2: Expand the WASM surface to full JovaWallet

**Files:**
- Modify: `crates/jova-core-wasm/src/lib.rs`

- [ ] **Step 1: Replace the Phase 1 stub with the full surface**

`crates/jova-core-wasm/src/lib.rs`:

```rust
//! jova-core-wasm — wasm-bindgen bindings for the public JovaWallet API.

#![forbid(unsafe_code)]

use jova_core::{
    JovaWallet as InnerWallet, JovaChain, JovaError, Strength,
    Mnemonic as CoreMnemonic, UnsignedTx, SignableMessage, Address, SignedTx, Signature,
};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen as swb;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn _start() {
    // Hook for browser-side panic-to-console mapping; useful for debug builds.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}

// ---------- Free functions ----------

#[wasm_bindgen(js_name = createMnemonic)]
pub fn create_mnemonic(bits256: bool) -> JsValue {
    let s = if bits256 { Strength::Bits256 } else { Strength::Bits128 };
    let m = jova_core::create_mnemonic(s);
    swb::to_value(&m).unwrap()
}

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}

#[wasm_bindgen(js_name = isValidAddress)]
pub fn is_valid_address(addr: &str, chain_js: JsValue) -> Result<bool, JsValue> {
    let chain: JovaChain = swb::from_value(chain_js).map_err(jserr)?;
    Ok(jova_core::is_valid_address(addr, &chain))
}

// ---------- Wallet object ----------

#[wasm_bindgen]
pub struct JovaWallet {
    inner: Option<InnerWallet>,   // Some until destroy(); None after.
}

#[wasm_bindgen]
impl JovaWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(mnemonic_js: JsValue) -> Result<JovaWallet, JsValue> {
        let m: CoreMnemonic = swb::from_value(mnemonic_js).map_err(jserr)?;
        let inner = InnerWallet::from_mnemonic(&m.words, &m.passphrase).map_err(jova_err)?;
        Ok(JovaWallet { inner: Some(inner) })
    }

    #[wasm_bindgen(js_name = destroy)]
    pub fn destroy(&mut self) {
        // Drops the inner wallet, which zeroizes via Drop.
        self.inner = None;
    }

    #[wasm_bindgen(js_name = address)]
    pub fn address(&self, chain_js: JsValue, account: u32) -> Result<JsValue, JsValue> {
        let inner = self.inner.as_ref().ok_or_else(|| js_err("wallet destroyed"))?;
        let chain: JovaChain = swb::from_value(chain_js).map_err(jserr)?;
        let a: Address = inner.address(&chain, account).map_err(jova_err)?;
        swb::to_value(&a).map_err(jserr)
    }

    #[wasm_bindgen(js_name = signTx)]
    pub fn sign_tx(&self, unsigned_js: JsValue) -> Result<JsValue, JsValue> {
        let inner = self.inner.as_ref().ok_or_else(|| js_err("wallet destroyed"))?;
        let unsigned: UnsignedTx = swb::from_value(unsigned_js).map_err(jserr)?;
        let signed: SignedTx = inner.sign_tx(&unsigned).map_err(jova_err)?;
        swb::to_value(&signed).map_err(jserr)
    }

    #[wasm_bindgen(js_name = signMessage)]
    pub fn sign_message(&self, msg_js: JsValue) -> Result<JsValue, JsValue> {
        let inner = self.inner.as_ref().ok_or_else(|| js_err("wallet destroyed"))?;
        let msg: SignableMessage = swb::from_value(msg_js).map_err(jserr)?;
        let sig: Signature = inner.sign_message(&msg).map_err(jova_err)?;
        swb::to_value(&sig).map_err(jserr)
    }
}

// ---------- Error mapping ----------

fn jserr<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&format!("{}", e))
}

fn js_err(s: &str) -> JsValue {
    JsValue::from_str(s)
}

#[derive(Serialize)]
struct JsErrorPayload<'a> {
    kind: &'a str,
    reason: Option<String>,
    chain: Option<String>,
}

fn jova_err(e: JovaError) -> JsValue {
    let payload = match &e {
        JovaError::InvalidMnemonic                    => JsErrorPayload { kind: "invalidMnemonic", reason: None, chain: None },
        JovaError::InvalidPassphrase                  => JsErrorPayload { kind: "invalidPassphrase", reason: None, chain: None },
        JovaError::InvalidAddress { chain }           => JsErrorPayload { kind: "invalidAddress", reason: None, chain: Some(chain.clone()) },
        JovaError::UnsupportedChain(s)                => JsErrorPayload { kind: "unsupportedChain", reason: None, chain: Some(s.clone()) },
        JovaError::MalformedUnsignedTx { reason }     => JsErrorPayload { kind: "malformedUnsignedTx", reason: Some(reason.clone()), chain: None },
        JovaError::MalformedSignableMessage { reason }=> JsErrorPayload { kind: "malformedSignableMessage", reason: Some(reason.clone()), chain: None },
        JovaError::SigningFailed { reason }           => JsErrorPayload { kind: "signingFailed", reason: Some(reason.clone()), chain: None },
        JovaError::Internal { reason }                => JsErrorPayload { kind: "internal", reason: Some(reason.clone()), chain: None },
    };
    swb::to_value(&payload).unwrap_or_else(|_| JsValue::from_str(&e.to_string()))
}
```

Add `console_error_panic_hook = "0.1"` to `crates/jova-core-wasm/Cargo.toml` for debug ergonomics.

- [ ] **Step 2: Build & smoke**

```bash
just build-wasm
ls bindings/wasm/pkg/
```

Expected: `jova_core_wasm.js`, `jova_core_wasm.d.ts`, `jova_core_wasm_bg.wasm`.

- [ ] **Step 3: Commit**

```bash
git add crates/jova-core-wasm/
git commit -m "feat(wasm): full JovaWallet surface via wasm-bindgen"
```

---

## Task 3: TypeScript types refinement

**Files:**
- Create: `bindings/wasm/src/types.ts`
- Modify: `bindings/wasm/src/index.ts`

- [ ] **Step 1: Hand-written discriminated-union types**

`bindings/wasm/src/types.ts`:

```typescript
export interface Mnemonic {
    words: string;
    passphrase: string;
}

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

export interface AccessListItem {
    address: string;
    storageKeys: string[];
}

export interface EvmUnsigned {
    chainId: bigint;
    nonce: bigint;
    to: string;
    value: string;
    gasLimit: bigint;
    maxFeePerGas: string;
    maxPriorityFeePerGas: string;
    data: string;
    accessList: AccessListItem[];
}

export type UnsignedTx =
    | { kind: 'evm'; tx: EvmUnsigned }
    | { kind: 'bitcoin'; psbtBase64: string }
    | { kind: 'solana'; messageBase64: string; recentBlockhash: string }
    | { kind: 'xrp'; txJson: string };

export type BtcMsgScheme = 'bip322' | 'legacy';

export type SignableMessage =
    | { kind: 'evmPersonalSign'; message: string }
    | { kind: 'evmTypedDataV4'; json: string }
    | { kind: 'solana'; messageBase64: string }
    | { kind: 'bitcoin'; message: string; address: string; scheme: BtcMsgScheme };

export interface Address {
    chain: JovaChain;
    value: string;
}

export interface SignedTx {
    chain: JovaChain;
    rawHex: string;
    txHash: string;
}

export interface Signature {
    hex: string;
}

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
```

- [ ] **Step 2: Re-export from index.ts with Disposable wrapper**

`bindings/wasm/src/index.ts`:

```typescript
import init, {
    JovaWallet as RawWallet,
    createMnemonic as rawCreateMnemonic,
    isValidMnemonic,
    isValidAddress,
} from '../pkg/jova_core_wasm.js';

import type {
    Mnemonic, JovaChain, UnsignedTx, SignableMessage,
    Address, SignedTx, Signature, JovaErrorPayload,
} from './types.js';

export type { Mnemonic, JovaChain, UnsignedTx, SignableMessage,
              Address, SignedTx, Signature, JovaErrorPayload };
export { isValidMnemonic, isValidAddress, init };
export { JovaException } from './types.js';

import { JovaException } from './types.js';

export function createMnemonic(bits256: boolean = false): Mnemonic {
    return rawCreateMnemonic(bits256) as Mnemonic;
}

/**
 * A Jova signing wallet. Implements `Disposable` (TypeScript 5.5+) so
 * `using wallet = JovaWallet.fromMnemonic(...)` automatically destroys
 * the underlying WASM-side seed buffer when the scope exits.
 *
 * Without `using`, you MUST call `wallet.destroy()` manually — the JS GC
 * does not run finalizers synchronously enough for crypto-grade clearing.
 */
export class JovaWallet implements Disposable {
    private constructor(private raw: RawWallet) {}

    static fromMnemonic(mnemonic: Mnemonic): JovaWallet {
        try {
            return new JovaWallet(new RawWallet(mnemonic));
        } catch (e) {
            throw asJovaException(e);
        }
    }

    address(chain: JovaChain, account: number = 0): Address {
        try {
            return this.raw.address(chain, account) as Address;
        } catch (e) {
            throw asJovaException(e);
        }
    }

    signTx(unsigned: UnsignedTx): SignedTx {
        try {
            return this.raw.signTx(unsigned) as SignedTx;
        } catch (e) {
            throw asJovaException(e);
        }
    }

    signMessage(msg: SignableMessage): Signature {
        try {
            return this.raw.signMessage(msg) as Signature;
        } catch (e) {
            throw asJovaException(e);
        }
    }

    destroy(): void {
        this.raw.destroy();
    }

    [Symbol.dispose](): void {
        this.destroy();
    }
}

function asJovaException(e: unknown): JovaException {
    if (typeof e === 'object' && e !== null && 'kind' in e) {
        return new JovaException(e as JovaErrorPayload);
    }
    return new JovaException({ kind: 'internal', reason: String(e) });
}
```

- [ ] **Step 3: Commit**

```bash
git add bindings/wasm/src/
git commit -m "feat(wasm/ts): typed surface with Disposable JovaWallet"
```

---

## Task 4: Per-chain entrypoints (tree-shaking)

**Files:**
- Create: `bindings/wasm/src/evm.ts`
- Create: `bindings/wasm/src/btc.ts`
- Create: `bindings/wasm/src/sol.ts`
- Create: `bindings/wasm/src/xrp.ts`
- Modify: `bindings/wasm/package.json` (add subpath exports)

- [ ] **Step 1: Per-chain wrappers**

`bindings/wasm/src/evm.ts`:

```typescript
import { JovaWallet, init, isValidAddress } from './index.js';
import type { Mnemonic, JovaChain, EvmUnsigned, UnsignedTx, SignableMessage, SignedTx, Signature, Address } from './types.js';

export { init, JovaWallet };
export type { Mnemonic, EvmUnsigned, SignedTx, Signature, Address };

export const EVM_CHAINS = {
    ethereum: { kind: 'ethereum' as const },
    polygon:  { kind: 'polygon'  as const },
    bsc:      { kind: 'bsc'      as const },
    arbitrum: { kind: 'arbitrum' as const },
    optimism: { kind: 'optimism' as const },
    base:     { kind: 'base'     as const },
    customEvm: (chainId: bigint): JovaChain => ({ kind: 'customEvm', chainId }),
};

export function isValidEvmAddress(addr: string): boolean {
    return isValidAddress(addr, EVM_CHAINS.ethereum);
}

export function evmTransfer(opts: {
    chainId: bigint; nonce: bigint; to: string; valueWei: string;
    gasLimit?: bigint; maxFeePerGas: string; maxPriorityFeePerGas: string;
}): UnsignedTx {
    return {
        kind: 'evm',
        tx: {
            chainId: opts.chainId,
            nonce: opts.nonce,
            to: opts.to,
            value: opts.valueWei,
            gasLimit: opts.gasLimit ?? 21_000n,
            maxFeePerGas: opts.maxFeePerGas,
            maxPriorityFeePerGas: opts.maxPriorityFeePerGas,
            data: '0x',
            accessList: [],
        },
    };
}
```

Same shape for `btc.ts`, `sol.ts`, `xrp.ts` — each exports its chain constants and helper builders.

- [ ] **Step 2: package.json subpath exports**

```json
{
  "name": "@jovachain/wallet-core",
  "version": "1.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": { "import": "./dist/index.js", "types": "./dist/index.d.ts" },
    "./evm": { "import": "./dist/evm.js", "types": "./dist/evm.d.ts" },
    "./btc": { "import": "./dist/btc.js", "types": "./dist/btc.d.ts" },
    "./sol": { "import": "./dist/sol.js", "types": "./dist/sol.d.ts" },
    "./xrp": { "import": "./dist/xrp.js", "types": "./dist/xrp.d.ts" },
    "./init": { "import": "./pkg/jova_core_wasm.js", "types": "./pkg/jova_core_wasm.d.ts" }
  },
  "files": ["dist/", "pkg/", "README.md", "COVERAGE.md"],
  "scripts": {
    "build": "./scripts/build-wasm.sh && tsc -p tsconfig.json",
    "test": "vitest run",
    "size-check": "node scripts/size-check.mjs"
  },
  "devDependencies": {
    "vitest": "^2.1.0",
    "typescript": "^5.5.0"
  }
}
```

(Tree-shaking is enabled because each entrypoint imports only the helpers it uses; bundlers strip unreferenced imports. The WASM blob is shared — only one copy ends up in any given consumer's bundle since the module identity is preserved via the `init` subpath.)

- [ ] **Step 3: Commit**

```bash
git add bindings/wasm/
git commit -m "feat(wasm/ts): per-chain entrypoints for tree-shaking"
```

---

## Task 5: Bundle size measurement in CI

**Files:**
- Create: `bindings/wasm/scripts/size-check.mjs`
- Modify: `.github/workflows/ci-bindings-wasm.yml`

- [ ] **Step 1: Size budget script**

`bindings/wasm/scripts/size-check.mjs`:

```javascript
import { readFileSync, statSync } from 'node:fs';
import { gzipSync } from 'node:zlib';

const BUDGETS = {
    'pkg/jova_core_wasm_bg.wasm': 2_000_000,    // 2 MB gzipped
    'dist/index.js':              200_000,       // 200 KB raw
};

let failed = false;
for (const [path, budget] of Object.entries(BUDGETS)) {
    const raw = readFileSync(path);
    const gz = gzipSync(raw);
    const sz = path.endsWith('.wasm') ? gz.length : raw.length;
    const status = sz <= budget ? '✅' : '❌';
    console.log(`${status} ${path}: ${sz} bytes (budget ${budget})`);
    if (sz > budget) failed = true;
}
process.exit(failed ? 1 : 0);
```

- [ ] **Step 2: Wire into CI**

Modify `.github/workflows/ci-bindings-wasm.yml`'s `wasm` job, append after the `pnpm test` step:

```yaml
      - run: cd bindings/wasm && pnpm run build && pnpm run size-check
```

- [ ] **Step 3: Commit**

```bash
git add bindings/wasm/scripts/ .github/workflows/ci-bindings-wasm.yml
git commit -m "ci(wasm): bundle-size budget check"
```

---

## Task 6: Web Worker example app

**Files:**
- Create: `examples/web-sample/package.json`
- Create: `examples/web-sample/vite.config.ts`
- Create: `examples/web-sample/index.html`
- Create: `examples/web-sample/src/main.ts`
- Create: `examples/web-sample/src/wallet-worker.ts`
- Create: `examples/web-sample/README.md`

- [ ] **Step 1: Worker — runs the SDK off the main thread**

`examples/web-sample/src/wallet-worker.ts`:

```typescript
import init, { JovaWallet, EVM_CHAINS, evmTransfer } from '@jovachain/wallet-core/evm';
import type { Mnemonic, UnsignedTx, SignedTx } from '@jovachain/wallet-core';

let wallet: JovaWallet | null = null;

self.addEventListener('message', async (e: MessageEvent) => {
    const { id, type, payload } = e.data;
    try {
        switch (type) {
            case 'init':
                await init();
                wallet = JovaWallet.fromMnemonic(payload as Mnemonic);
                self.postMessage({ id, ok: true });
                break;

            case 'address':
                if (!wallet) throw new Error('not initialized');
                self.postMessage({ id, ok: true, result: wallet.address(payload.chain, payload.account ?? 0) });
                break;

            case 'sign':
                if (!wallet) throw new Error('not initialized');
                self.postMessage({ id, ok: true, result: wallet.signTx(payload as UnsignedTx) });
                break;

            case 'destroy':
                wallet?.destroy();
                wallet = null;
                self.postMessage({ id, ok: true });
                self.close();
                break;
        }
    } catch (err) {
        self.postMessage({ id, ok: false, error: String(err) });
    }
});
```

- [ ] **Step 2: Main thread driver**

`examples/web-sample/src/main.ts`:

```typescript
const worker = new Worker(new URL('./wallet-worker.ts', import.meta.url), { type: 'module' });

let nextId = 0;
function call(type: string, payload?: any): Promise<any> {
    const id = nextId++;
    return new Promise((resolve, reject) => {
        const handler = (e: MessageEvent) => {
            if (e.data.id !== id) return;
            worker.removeEventListener('message', handler);
            e.data.ok ? resolve(e.data.result) : reject(new Error(e.data.error));
        };
        worker.addEventListener('message', handler);
        worker.postMessage({ id, type, payload });
    });
}

(async () => {
    const mnemonic = {
        words: 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about',
        passphrase: '',
    };
    await call('init', mnemonic);
    const eth = await call('address', { chain: { kind: 'ethereum' } });
    document.getElementById('eth-address')!.textContent = eth.value;

    const polygon = await call('address', { chain: { kind: 'polygon' } });
    document.getElementById('polygon-address')!.textContent = polygon.value;

    document.getElementById('sign-button')!.addEventListener('click', async () => {
        const tx = {
            kind: 'evm',
            tx: {
                chainId: 1n, nonce: 0n,
                to: '0x0000000000000000000000000000000000000000',
                value: '1000000000000000000',
                gasLimit: 21000n,
                maxFeePerGas: '30000000000',
                maxPriorityFeePerGas: '2000000000',
                data: '0x',
                accessList: [],
            },
        };
        const signed = await call('sign', tx);
        document.getElementById('signed-tx')!.textContent = signed.rawHex;
    });

    window.addEventListener('beforeunload', () => { call('destroy'); });
})();
```

- [ ] **Step 3: Vite config + HTML**

Standard Vite + TypeScript boilerplate. Browse at `pnpm dev`; visit `localhost:5173`. Verify ETH address renders, Polygon address renders (same value — proves EVM family share derivation), Sign button produces a hex string.

- [ ] **Step 4: README**

`examples/web-sample/README.md`:

```markdown
# Web Sample

Minimal browser app showing the recommended Jova WASM consumption pattern:
- WASM runs in a Web Worker, isolated from the page's JS context.
- Mnemonic and wallet handle live in the worker.
- Main thread dispatches signing requests via postMessage.

## Run
```
pnpm install
pnpm dev
```
```

- [ ] **Step 5: Commit**

```bash
git add examples/web-sample/
git commit -m "examples: web-sample with Web Worker + WASM"
```

---

## Task 7: Vector parity tests via vitest

**Files:**
- Create: `bindings/wasm/tests/vectors-evm.test.ts`
- Create: `bindings/wasm/tests/vectors-btc.test.ts`
- Create: `bindings/wasm/tests/vectors-sol.test.ts`
- Create: `bindings/wasm/tests/vectors-xrp.test.ts`
- Create: `bindings/wasm/tests/decoders.ts`

- [ ] **Step 1: Decoder helpers (vector JSON → typed objects)**

`bindings/wasm/tests/decoders.ts`:

```typescript
import type { JovaChain, UnsignedTx, EvmUnsigned, SignableMessage } from '@jovachain/wallet-core';

export function decodeChain(o: any): JovaChain {
    switch (o.kind) {
        case 'ethereum': case 'polygon': case 'bsc': case 'arbitrum':
        case 'optimism': case 'base': case 'bitcoin': case 'solana':
        case 'xrp':
            return { kind: o.kind };
        case 'customEvm':
            return { kind: 'customEvm', chainId: BigInt(o.chainId) };
        default:
            throw new Error(`unknown chain kind: ${o.kind}`);
    }
}

export function decodeUnsigned(o: any): UnsignedTx {
    switch (o.kind) {
        case 'evm':
            return { kind: 'evm', tx: decodeEvm(o) };
        case 'bitcoin':
            return { kind: 'bitcoin', psbtBase64: o.psbtBase64 };
        case 'solana':
            return { kind: 'solana', messageBase64: o.messageBase64, recentBlockhash: o.recentBlockhash };
        case 'xrp':
            return { kind: 'xrp', txJson: o.txJson };
        default:
            throw new Error(`unknown unsigned kind: ${o.kind}`);
    }
}

export function decodeEvm(o: any): EvmUnsigned {
    return {
        chainId: BigInt(o.chainId),
        nonce: BigInt(o.nonce),
        to: o.to,
        value: o.value,
        gasLimit: BigInt(o.gasLimit),
        maxFeePerGas: o.maxFeePerGas,
        maxPriorityFeePerGas: o.maxPriorityFeePerGas,
        data: o.data,
        accessList: (o.accessList ?? []).map((i: any) => ({
            address: i.address,
            storageKeys: i.storageKeys,
        })),
    };
}
```

- [ ] **Step 2: EVM vector test**

`bindings/wasm/tests/vectors-evm.test.ts`:

```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import init, { JovaWallet } from '../src/index.js';
import vectors from '../../../spec/test-vectors.json';
import { decodeChain, decodeUnsigned } from './decoders.js';

beforeAll(async () => { await init(); });

describe('EVM vectors', () => {
    const evmKinds = ['ethereum', 'polygon', 'bsc', 'arbitrum', 'optimism', 'base', 'customEvm'];

    for (const v of vectors.vectors) {
        if (v.kind !== 'address') continue;
        if (!evmKinds.includes(v.input.chain.kind)) continue;
        it(`address: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const got = wallet.address(decodeChain(v.input.chain), 0);
                expect(got.value).toBe(v.expected.address);
            } finally { wallet.destroy(); }
        });
    }

    for (const v of vectors.vectors) {
        if (v.kind !== 'sign_tx') continue;
        if (v.input.unsigned_tx.kind !== 'evm') continue;
        it(`sign_tx: ${v.id}`, () => {
            const wallet = JovaWallet.fromMnemonic({
                words: v.input.mnemonic,
                passphrase: v.input.passphrase ?? '',
            });
            try {
                const signed = wallet.signTx(decodeUnsigned(v.input.unsigned_tx));
                expect(signed.rawHex.toLowerCase()).toBe(v.expected.signed_hex.toLowerCase());
                expect(signed.txHash.toLowerCase()).toBe(v.expected.tx_hash.toLowerCase());
            } finally { wallet.destroy(); }
        });
    }
});
```

- [ ] **Step 3: BTC / SOL / XRP test files**

Same shape, different filters. For chains that are WASM-flagged-off (per `COVERAGE.md`), skip the test file with `describe.skip` or guard with a feature check at top.

- [ ] **Step 4: Run**

```bash
just build-wasm
cd bindings/wasm && pnpm install && pnpm test
```

Expected: every vector that applies to a WASM-supported chain passes byte-identically with Rust + Swift + Kotlin.

- [ ] **Step 5: Commit**

```bash
git add bindings/wasm/tests/
git commit -m "test(wasm): vector parity for every WASM-supported chain"
```

---

## Task 8: npm publish setup

**Files:**
- Modify: `bindings/wasm/package.json` (publish metadata)
- Modify: `.github/workflows/release.yml` (npm step)

- [ ] **Step 1: Verify package metadata**

`bindings/wasm/package.json` should have:

```json
{
  "name": "@jovachain/wallet-core",
  "version": "1.1.0",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/jovachain/jovawallet-core.git",
    "directory": "bindings/wasm"
  },
  "keywords": ["bitcoin", "ethereum", "solana", "xrp", "wallet", "signing", "wasm"],
  "publishConfig": { "access": "public" }
}
```

- [ ] **Step 2: Stage on npm with `@rc` dist-tag**

The release pipeline from Phase 5's `release.yml` already handles npm publish for non-RC tags. For Phase 6's first publish, run the existing pipeline on a `v1.1.0-rc.1` tag, smoke-test the staging artifact, then tag `v1.1.0`.

- [ ] **Step 3: Smoke test consumption from a fresh project**

```bash
cd /tmp
mkdir wasm-smoke && cd wasm-smoke
pnpm init
pnpm add @jovachain/wallet-core@rc
cat > test.mjs <<'EOF'
import init, { JovaWallet, isValidMnemonic } from '@jovachain/wallet-core';
await init();
console.log(isValidMnemonic('not a valid mnemonic', '') === false ? 'OK' : 'FAIL');
EOF
node test.mjs
```

Expected: `OK`.

- [ ] **Step 4: Commit any package.json adjustments**

```bash
git add bindings/wasm/package.json
git commit -m "chore(wasm): publish metadata for @jovachain/wallet-core"
```

---

## Task 9: Update integration-web.md with real coverage table

**Files:**
- Modify: `docs/integration-web.md`

- [ ] **Step 1: Replace the "honest disclaimer" with real coverage**

The disclaimer added in Phase 0 / 1 ("WASM coverage may lag native if a chain crate is uncooperative") was correct *as of Phase 0–5*. After Phase 6 lands, replace it with the real coverage table from `bindings/wasm/COVERAGE.md`. If a chain ended up flagged-off, document the gap and the workaround (use the native binding or backend Rust).

- [ ] **Step 2: Add Web Worker pattern reference**

Reference `examples/web-sample/` as the canonical browser consumption pattern. Document the `using` syntax and the manual `destroy()` fallback.

- [ ] **Step 3: Commit**

```bash
git add docs/integration-web.md
git commit -m "docs(web): replace disclaimer with real coverage; reference web-sample"
```

---

## Task 10: Open PR, RC, tag v1.1.0

- [ ] **Step 1: Open PR**

```bash
git push -u origin feat/phase-6-wasm
gh pr create --title "Phase 6: WASM functional + npm publish" --body "$(cat <<'EOF'
## Summary
- Full JovaWallet surface in jova-core-wasm via wasm-bindgen
- Typed TS surface with discriminated unions; JovaWallet implements Disposable
- Per-chain entrypoints (@jovachain/wallet-core/evm, /btc, /sol, /xrp)
- examples/web-sample with Web Worker pattern
- Vector parity passing for every WASM-supported chain
- Bundle-size budget enforced in CI
- npm publish wired into release.yml; @jovachain/wallet-core@1.1.0 ready

## Test plan
- [x] just build-wasm + pnpm test passes
- [x] Bundle size within budget (gzip < 2 MB full, < 500 kB EVM-only)
- [x] examples/web-sample runs end-to-end in browser
- [x] Smoke install of @rc dist-tag in fresh project works
- [x] Coverage doc reflects feasibility-report findings

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 2: After CI green, tag RC and run release pipeline in dry-run**

```bash
git tag v1.1.0-rc.1
git push origin v1.1.0-rc.1
gh workflow view release.yml
# Watch the workflow stage Maven (no-op for non-Rust changes), publish npm @rc, etc.
```

- [ ] **Step 3: Smoke test the @rc staging**

```bash
cd /tmp && rm -rf wasm-smoke && mkdir wasm-smoke && cd wasm-smoke
pnpm init -y
pnpm add @jovachain/wallet-core@rc
cat > test.mjs <<'EOF'
import init, { JovaWallet, createMnemonic } from '@jovachain/wallet-core';
await init();
const m = createMnemonic(false);
console.log('words count:', m.words.split(' ').length);
const w = JovaWallet.fromMnemonic(m);
const eth = w.address({ kind: 'ethereum' }, 0);
console.log('eth:', eth.value);
w.destroy();
EOF
node test.mjs
```

Expected: 12 words, valid ETH address.

- [ ] **Step 4: Tag v1.1.0**

```bash
git tag -a v1.1.0 -m "v1.1.0 — Phase 6 WASM functional"
git push origin v1.1.0
```

The release pipeline auto-promotes the npm `@rc` dist-tag to `latest`.

---

## Self-review

- [ ] Every task has exact paths and commands.
- [ ] WASM surface mirrors the Rust + uniffi surface (createMnemonic, isValidMnemonic, isValidAddress, JovaWallet with from-mnemonic / address / signTx / signMessage / destroy).
- [ ] TypeScript types are discriminated unions with proper bigint usage for u64 fields.
- [ ] JovaWallet implements Symbol.dispose for `using` syntax.
- [ ] Per-chain entrypoints enable tree-shaking.
- [ ] Bundle-size budget is enforced in CI.
- [ ] Web Worker example is the documented consumption pattern.
- [ ] Vector parity tests cover every WASM-supported chain.
- [ ] Coverage doc honest about any chain that's WASM-flagged-off.
- [ ] npm publish smoke-tested from a fresh project.

---

## What this plan does NOT do

- React Native binding via `uniffi-bindgen-react-native` — Phase 7+ candidate.
- viem / wagmi adapter package — separate repo.
- Service-worker-based key custody (advanced web wallet pattern) — out of scope.
- Solana SPL token signing helpers — apps build SPL transfers; SDK signs whatever message they produce.

---

## Estimated time

2–3 weeks for a senior team. Time sinks:
1. TypeScript discriminated-union plumbing across the type boundaries.
2. Web Worker postMessage protocol design (structured clone limits, transferables).
3. npm dist-tag staging and smoke-test reliability.
4. Bundle-size optimization if the budget is tight (most likely battle: stripping `solana-transaction`'s deps).
