# Release checksums

SHA-256 hashes of the canonical artifacts for each tagged release. Two engineers, each on a separate machine, build from the same Git SHA and confirm every hash matches before signing off on the release.

This is the reproducible-build gate required for `v1.0.0` per [`docs/superpowers/plans/2026-05-05-phase-5-hardening-audit-v1.md`](superpowers/plans/2026-05-05-phase-5-hardening-audit-v1.md) §4.

## How to verify

```bash
git checkout <tag>
cargo build --workspace --release
./bindings/swift/scripts/build-xcframework.sh         # macOS only
./bindings/kotlin/scripts/build-aar.sh
./bindings/wasm/scripts/build-wasm.sh

shasum -a 256 target/release/libjova_core_ffi.a
shasum -a 256 bindings/swift/JovaCore.xcframework/*/libjova_core_ffi.a   # macOS only
shasum -a 256 bindings/wasm/pkg/jova_core_wasm_bg.wasm
shasum -a 256 bindings/kotlin/jova-core/build/outputs/aar/*.aar
```

Compare each line against the canonical hashes below.

If a hash differs, look for non-deterministic build inputs:
- Build timestamp leaking through `built` / `vergen` / similar crates.
- Codegen unit randomness — should be pinned via `codegen-units = 1` in release profile (already set).
- Random per-build IDs in the Rust compiler. With `CARGO_TARGET_DIR` pointing at a clean tree and `RUSTFLAGS` empty, output should be stable.

## v1.0.0 (TBD)

Awaiting Phase 5 completion. Engineers signing off:

- Engineer A: _TBD_, machine: _TBD_, host platform: _TBD_
- Engineer B: _TBD_, machine: _TBD_, host platform: _TBD_

### Artifacts

| Artifact | SHA-256 |
|---|---|
| `target/release/libjova_core_ffi.a` (Linux x86_64) | _TBD_ |
| `target/aarch64-linux-android/release/libjova_core_ffi.so` | _TBD_ |
| `target/armv7-linux-androideabi/release/libjova_core_ffi.so` | _TBD_ |
| `target/x86_64-linux-android/release/libjova_core_ffi.so` | _TBD_ |
| `target/i686-linux-android/release/libjova_core_ffi.so` | _TBD_ |
| `bindings/swift/JovaCore.xcframework/.../libjova_core_ffi.a` (iOS device) | _TBD_ |
| `bindings/swift/JovaCore.xcframework/.../libjova_core_ffi.a` (iOS simulator) | _TBD_ |
| `bindings/swift/JovaCore.xcframework/.../libjova_core_ffi.a` (macOS) | _TBD_ |
| `bindings/wasm/pkg/jova_core_wasm_bg.wasm` | _TBD_ |
| `bindings/kotlin/jova-core/build/outputs/aar/jova-core-release.aar` | _TBD_ |

## Prior releases

(None yet. v0.0.1 through v0.3.0 predated this gate.)
