import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        // wasm-pack --target nodejs emits CJS; the test runs in Node's vm
        // context where synchronous require() is available.
        environment: 'node',
        // Allow importing CJS modules from the pkg/ directory.
        server: {
            deps: {
                // Treat the wasm-pack generated CJS bundle as external so
                // vitest's vite pipeline doesn't try to ESM-transform it.
                external: ['../pkg/jova_core_wasm.js', './pkg/jova_core_wasm.js'],
            },
        },
    },
});
