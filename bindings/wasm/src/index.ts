// Re-export from the wasm-pack generated package (--target nodejs).
// In nodejs mode, the wasm is loaded synchronously at require() time — no
// async init() call is needed, unlike the --target web variant.
import { isValidMnemonic } from '../pkg/jova_core_wasm.js';
export { isValidMnemonic };
