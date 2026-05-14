// Bundle-size budget check for @jovachain/wallet-core.
//
// Run after `./scripts/build-wasm.sh` to verify the produced artifacts stay
// within the v1.1 budgets.  CI invokes this via `pnpm run size-check`.
//
// Budgets are gzipped sizes for the .wasm (the wire-format consumers serve
// to browsers) and raw byte size for the JS shim.

import { readFileSync, existsSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..');

/**
 * The Rust core compiled to WASM is the dominant artifact.  2 MB gzip is the
 * v1.1 budget per the plan; current size is ~800 KB so there is plenty of
 * headroom.  The wasm-pack JS shim is tiny (~20 KB) but still budgeted to
 * catch future bloat.
 */
const BUDGETS = {
    'pkg/jova_core_wasm_bg.wasm': { kind: 'gzip', max: 2_000_000 },
    'pkg/jova_core_wasm.js':      { kind: 'raw',  max:   200_000 },
};

let failed = false;
for (const [rel, { kind, max }] of Object.entries(BUDGETS)) {
    const path = resolve(root, rel);
    if (!existsSync(path)) {
        console.error(`MISSING ${rel} — did you run scripts/build-wasm.sh?`);
        failed = true;
        continue;
    }
    const raw = readFileSync(path);
    const size = kind === 'gzip' ? gzipSync(raw).length : raw.length;
    const ok = size <= max;
    const status = ok ? 'OK ' : 'OVER';
    const measured = kind === 'gzip' ? `${size} bytes gzipped` : `${size} bytes`;
    console.log(`${status} ${rel}: ${measured} (budget ${max})`);
    if (!ok) failed = true;
}

process.exit(failed ? 1 : 0);
