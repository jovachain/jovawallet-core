// Note: wasm-pack --target web output uses fetch() which Node does not support
// for file:// URLs. This smoke test uses the --target nodejs build instead.
// Both targets produce the same WASM bytes; the JS glue differs in how it loads
// the .wasm file (fetch vs fs.readFileSync).
import { ping_wasm } from '../generated/wasm-baseline-node/jova_spike_wasm.js';
const result = ping_wasm();
console.log(result);
if (result !== 'pong-wasm') {
    console.error('FAIL: expected pong-wasm');
    process.exit(1);
}
console.log('[ok] WASM round-trip works');
