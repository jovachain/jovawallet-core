# Build and Release

How `jovawallet-core` is built, packaged, signed, and shipped — from a Git tag to consumable artifacts on every supported package manager.

## Versioning

Single semver number, lockstep across every binding. (ADR D8.)

- `vMAJOR.MINOR.PATCH`
- **MAJOR** — breaking change to the public API (removed/renamed method, removed chain, broken vector backward compat).
- **MINOR** — additive change (new chain, new method, new error variant, new optional field on `UnsignedTx`).
- **PATCH** — bugfix only. No public API change. Vector files may grow (new vectors that prior versions weren't tested against), never shrink.

Pre-release tags `vX.Y.Z-rc.N` for release candidates.

`v0.x` allows minor-version breakage. From `v1.0.0` onward the contract is locked.

---

## What we publish on every tag

| Artifact | Where | Identifier |
|---|---|---|
| Rust crates | crates.io | `jova-core`, `jova-core-primitives`, `jova-core-chains` |
| Swift package | satellite repo `jovachain/jovawallet-core-swift` | tag `vX.Y.Z`, SwiftPM consumption |
| Android library | Maven Central | `io.jovachain:jova-core:X.Y.Z` |
| JVM library | Maven Central | `io.jovachain:jova-core-jvm:X.Y.Z` (Phase 6+) |
| WASM library | npm | `@jovachain/wallet-core@X.Y.Z` (Phase 6+) |
| GitHub release | github.com/jovachain/jovawallet-core | binaries + checksums |

Every artifact is **built from the same Git commit**. The release workflow stages everything before publishing anything irreversible, then publishes in ordered, least-revertible-last sequence (Swift satellite → Maven → npm → crates.io). The aspiration is "every artifact at every tag" — there is no scenario where Swift sits at `1.4.2` and Kotlin at `1.4.3`. The reality, because package-manager rollback is uneven, is documented in the partial-publish recovery procedure later in this document.

---

## CI workflows

```
.github/workflows/
├── ci.yml                          run on push/PR; Rust tests on Linux/macOS/Windows
├── ci-bindings-swift.yml           run on push/PR; build XCFramework + run Swift tests
├── ci-bindings-kotlin.yml          run on push/PR; build AAR + run JUnit
├── ci-bindings-wasm.yml            run on push/PR; build npm package + run vitest
├── ci-no-std.yml                   run on push/PR; build primitives for thumbv7em-none-eabihf
├── nightly-fuzz.yml                cron 02:00 UTC; cargo-fuzz 30 min per target
├── nightly-miri.yml                cron 03:00 UTC; cargo miri test on jova-core-primitives
├── audit.yml                       run on push/PR + cron daily; cargo audit + deny + vet
└── release.yml                     run on tag; publish all artifacts
```

### `ci.yml` — Rust core

```yaml
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace --locked
      - run: cargo test --workspace --locked --release
```

### `ci-bindings-swift.yml`

Runs on macOS only.

1. Install Rust + iOS targets (`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`, `aarch64-apple-darwin`, `x86_64-apple-darwin`).
2. `cargo build --release` × 5 targets in `crates/jova-core-ffi`.
3. Run `uniffi-bindgen generate` to produce `JovaCore.swift` and the Module.modulemap.
4. `xcodebuild -create-xcframework` combining the static libs + headers into `JovaCore.xcframework`.
5. `cd bindings/swift && swift test` — runs `VectorsTests`, `ApiSurfaceTests`, etc., loading vectors from `../../spec/test-vectors.json`.
6. Upload XCFramework as a workflow artifact (used by `release.yml`).

### `ci-bindings-kotlin.yml`

Runs on Linux.

1. Install Rust + Android NDK + 4 Android targets (`aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`, `i686-linux-android`).
2. `cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 build --release` from `crates/jova-core-ffi`.
3. Run `uniffi-bindgen generate` to produce `JovaCore.kt`.
4. Drop `.so` files into `bindings/kotlin/jova-core/src/main/jniLibs/<abi>/`.
5. `cd bindings/kotlin && ./gradlew :jova-core:test` — runs `VectorsTest`, `ApiSurfaceTest`, etc.
6. `./gradlew :jova-core:assembleRelease` builds the AAR.
7. Upload AAR as a workflow artifact.

### `ci-bindings-wasm.yml`

Runs on Linux. Required from Phase 0; functional test coverage grows over time.

1. Install Rust + `wasm-pack` + `wasm32-unknown-unknown` target.
2. `wasm-pack build --release --target web` in `crates/jova-core-wasm`. **Compile smoke from Phase 0** — every PR proves the WASM target keeps building.
3. `cd bindings/wasm && pnpm install && pnpm test` — runs `vitest`. From Phase 0 the suite is a hello-world; from Phase 6 it's the full vector parity suite.
4. `pnpm build` produces the npm package.
5. Upload as a workflow artifact.

If a chain crate identified in Phase -1 does not compile to WASM, the WASM build feature-flags it off and the documentation flags the gap. The build itself never goes red — gaps are tracked, not silently skipped.

### `ci-no-std.yml`

```yaml
on: [push, pull_request]
jobs:
  no_std:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: thumbv7em-none-eabihf }
      - run: cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --no-default-features
```

If anyone accidentally adds a `std` dependency to primitives, this fails.

### `nightly-fuzz.yml`

```yaml
on:
  schedule: [{ cron: '0 2 * * *' }]
  workflow_dispatch:
jobs:
  fuzz:
    strategy:
      matrix:
        target:
          - fuzz_psbt_sign
          - fuzz_eip1559_decode
          - fuzz_eip712_typed
          - fuzz_sol_versioned_tx
          - fuzz_xrp_canonical
          - fuzz_mnemonic_parse
          - fuzz_path_parse
          - fuzz_address_parse
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - name: download-corpus
        run: gh repo clone jovachain/jovawallet-core-fuzz-corpus corpus
      - run: cargo fuzz run ${{ matrix.target }} corpus/${{ matrix.target }} -- -max_total_time=1800
      - name: upload-corpus
        if: success()
        run: |
          cd corpus && git add . && git commit -m "fuzz corpus update" && git push
      - name: file-issue-on-crash
        if: failure()
        uses: actions/github-script@v7
        with:
          script: |
            // create issue with reproducer attached
```

### `audit.yml`

```yaml
on:
  push:
  pull_request:
  schedule: [{ cron: '0 4 * * *' }]
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-audit cargo-deny cargo-vet
      - run: cargo audit
      - run: cargo deny check
      - run: cargo vet
```

`cargo-audit` flags RUSTSEC advisories. `cargo-deny` enforces our license whitelist + advisory denylist + the layered-dependency invariant. `cargo-vet` confirms our `supply-chain/` audits are still consistent.

### `release.yml` — staged publisher with RC support

Triggered by tags matching `v*`. The aspiration is "publish everything or nothing" but the reality is that package managers vary in how revertible they are. The workflow is designed around that asymmetry:

- **Stage everything that supports staging.** Validate before any irreversible step.
- **Publish in least-revertible-last order**: Swift satellite → Maven Central → npm → crates.io. If something breaks, it breaks in the most-revertible target.
- **RC tags run the full pipeline in dry-run.** A `vX.Y.Z-rc.N` tag exercises the entire workflow but ends at staging; no public publish.
- **Publish workflow is idempotent where possible.** Re-running after a transient failure must not double-publish.

#### Revertibility per registry (truth in 2026)

| Registry | What you *can* do post-publish | What you *cannot* |
|---|---|---|
| Swift satellite repo | Force-update tag, delete tag, delete release | Force consumers to drop cached `.swiftpm` data — they may have already resolved |
| Maven Central | `deprecate` a version with a notice | Delete an artifact. Once released from staging, it is permanent |
| npm | `deprecate` a version; `unpublish` only within 72h *and* only if no other package depends on it | Restore once unpublished if anyone depended on it |
| crates.io | `cargo yank` (prevents new uses, does not delete) | Delete a published version. Ever. |

This is why the publish order matters: if step N fails, every step before it is *partially recoverable* and every step after it never ran.

#### Workflow structure

```yaml
on:
  push:
    tags: ['v*']

jobs:
  preflight:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.parse.outputs.version }}
      is_rc: ${{ steps.parse.outputs.is_rc }}
    steps:
      - uses: actions/checkout@v4
      - id: parse
        run: |
          v="${GITHUB_REF#refs/tags/}"
          [[ "$v" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]] || { echo "bad tag"; exit 1; }
          echo "version=${v#v}" >> "$GITHUB_OUTPUT"
          [[ "$v" == *-rc.* ]] && echo "is_rc=true" >> "$GITHUB_OUTPUT" || echo "is_rc=false" >> "$GITHUB_OUTPUT"
      - run: cargo test --workspace --locked --release
      - run: ./tools/release/tag.sh   # validates Cargo.toml + package.json + build.gradle.kts versions match the tag

  build_swift:
    needs: preflight
    runs-on: macos-latest
    steps: [ … build XCFramework, upload artifact … ]

  build_kotlin:
    needs: preflight
    runs-on: ubuntu-latest
    steps: [ … build AAR + JVM jar, GPG-sign, upload artifact … ]

  build_wasm:
    needs: preflight
    runs-on: ubuntu-latest
    steps: [ … wasm-pack, npm pack --dry-run, upload artifact … ]

  # Stage 1: Maven OSSRH staging — closeable / dropable; no public publish yet.
  stage_maven:
    needs: build_kotlin
    runs-on: ubuntu-latest
    steps:
      - name: upload-to-ossrh-staging
        run: ./gradlew publishToSonatype closeSonatypeStagingRepository
      - name: smoke-test-staging
        run: ./tools/release/smoke-test-maven-staging.sh ${{ needs.preflight.outputs.version }}
        # Pulls the artifact from the staging URL, builds a tiny test app, runs it.

  # Stage 2: npm publish under -rc dist-tag (overridable later).
  stage_npm:
    needs: build_wasm
    runs-on: ubuntu-latest
    steps:
      - name: dry-run
        run: cd bindings/wasm && npm publish --dry-run --tag rc
      - if: needs.preflight.outputs.is_rc == 'true'
        name: publish-as-rc
        run: cd bindings/wasm && npm publish --tag rc
      # For non-RC tags, npm publish happens later in the publish_release job.

  # RC tags stop here. Smoke-test the staged artifacts manually, then tag the real version.
  rc_complete:
    needs: [stage_maven, stage_npm]
    if: needs.preflight.outputs.is_rc == 'true'
    runs-on: ubuntu-latest
    steps:
      - run: echo "RC ${{ needs.preflight.outputs.version }} staged. Smoke test, then tag the non-RC version."
      - name: drop-maven-staging
        run: ./gradlew dropSonatypeStagingRepository
        # The RC artifact in OSSRH staging is dropped; the real release will re-stage from a fresh tag.

  # Non-RC publish path: ordered, least-revertible-last.
  publish_release:
    needs: [stage_maven, stage_npm]
    if: needs.preflight.outputs.is_rc == 'false'
    runs-on: ubuntu-latest
    steps:
      # 1. Push to satellite Swift repo (force-recoverable).
      - name: publish-swift
        run: ./tools/release/publish-swift.sh ${{ needs.preflight.outputs.version }}

      # 2. Release Maven from staging (deprecation-recoverable).
      - name: release-maven
        run: ./gradlew releaseSonatypeStagingRepository

      # 3. Promote npm dist-tag from rc to latest (deprecation-recoverable).
      - name: promote-npm
        run: cd bindings/wasm && npm dist-tag add @jovachain/wallet-core@${{ needs.preflight.outputs.version }} latest

      # 4. crates.io publish — last because it's the least revertible.
      - name: publish-crates
        run: |
          cargo publish -p jova-core-primitives
          cargo publish -p jova-core-chains
          cargo publish -p jova-core

      # 5. GitHub release with checksums.
      - name: github-release
        run: gh release create "v${{ needs.preflight.outputs.version }}" \
          --notes-file CHANGELOG.md \
          ./dist/*.zip ./dist/*.aar ./dist/*.tgz ./dist/SHA256SUMS
```

### Partial-publish recovery procedure

If `publish_release` fails partway through, the runbook is:

#### If failure is at step 1 (Swift satellite push)

- Nothing else has published. Delete the satellite tag if it was created (`git -C satellite push origin :refs/tags/vX.Y.Z`).
- Investigate, fix, re-tag, re-run.

#### If failure is at step 2 (Maven release-from-staging)

- Swift is published. Decide: hold the swift release alone (consumers see it before others, but it'll work) or force-delete the satellite tag and roll forward later.
- The Maven staging repo can be `drop`'d to discard the staged artifact.
- Investigate the OSSRH error (most often: GPG signature, javadoc validation, checksum mismatch). Fix and re-run from `stage_maven` onward.

#### If failure is at step 3 (npm dist-tag promotion)

- Swift and Maven are published. The npm RC tag exists at `@rc` but `latest` was not updated.
- Apps that pin to `@latest` are still on the previous version — *this is the desired safe state*.
- Investigate npm error, retry the dist-tag command. This step is idempotent.

#### If failure is at step 4 (crates.io publish)

- Swift, Maven, npm are all published. crates.io is not.
- crates.io publish is least revertible *if it succeeds*. A failure means it didn't happen — retry directly. `cargo publish` is idempotent at the version level; re-running for an already-published version errors but doesn't break.
- If crates.io is genuinely unreachable (rare), wait and retry. There is no rollback for the other registries that's worth doing — they're at the right version, just one step ahead of crates.io.

#### If failure is at step 5 (GitHub release)

- Everything is published. `gh release create` failed (most often: rate limiting or auth).
- Retry manually. The release page is metadata — no consumer dependency on it.

#### When to roll forward, when to roll back

| Failure point | Default action |
|---|---|
| Anywhere before any public publish (preflight, builds, staging) | Fix and re-tag |
| Step 1 only (Swift) | Delete satellite tag, fix, re-tag |
| Step 2 (Maven) | Drop staging, fix, re-tag *higher version* if the Swift tag is in the wild |
| Step 3+ (npm or later) | Roll forward — release a `vX.Y.(Z+1)` patch with whatever fix is needed; deprecate the broken intermediate state |

Rolling **back** is harder than rolling **forward** for any registry past Maven. Default to forward.

### RC cycle for v1.0.0

For the v1.0 release specifically (and any major version):

1. Tag `v1.0.0-rc.1` → full pipeline runs, stops at `rc_complete`.
2. Smoke-test the staged Maven artifact via `tools/release/smoke-test-maven-staging.sh`, the npm RC dist-tag, and a private branch of the satellite Swift repo.
3. Smoke-test from app-team perspective: open a fresh iOS / Android / web sample project, install from staging, run vector tests.
4. If anything fails: drop staging, fix, tag `v1.0.0-rc.2`. Repeat.
5. When RC is clean: tag `v1.0.0`. Full publish runs.

For minor and patch releases, RC cycle is optional but recommended for any release that touches `jova-core-primitives` or the FFI layer.

---

## Satellite Swift repo

SwiftPM resolves binary-target packages best when `Package.swift` sits at the root of the repo it lives in. We respect that.

`jovachain/jovawallet-core-swift` is a *generated* repo. Humans never commit to it directly. On every release, `tools/release/publish-swift.sh`:

1. Clones `jovachain/jovawallet-core-swift`.
2. Drops the new `Package.swift` (referencing the latest XCFramework as a binary target with checksum).
3. Drops `JovaCore.xcframework.zip` as a release asset.
4. Drops the `Sources/JovaCore/Convenience.swift` ergonomics layer.
5. Commits with message `Release vX.Y.Z (auto-generated from jovachain/jovawallet-core@<sha>)`.
6. Tags `vX.Y.Z`.
7. Pushes.

Apps consume:

```swift
.package(url: "https://github.com/jovachain/jovawallet-core-swift.git", from: "1.4.2")
```

This pattern is identical to BDK's `bitcoindevkit/bdk-swift` repo.

---

## Maven Central credentials

Stored as GitHub Actions secrets:

- `OSSRH_USERNAME` / `OSSRH_TOKEN` — Sonatype credentials.
- `GPG_PRIVATE_KEY` / `GPG_PASSPHRASE` — for signing the published artifacts (Maven Central requires GPG signatures).

Rotated every 6 months.

---

## crates.io credentials

`CARGO_REGISTRY_TOKEN` GitHub Actions secret. Scoped to a CI-only token that can publish but not yank or unyank.

---

## npm credentials

`NPM_TOKEN` GitHub Actions secret. Scoped to publish-only on the `@jovachain` scope.

---

## Reproducible builds

We aim for deterministic, byte-identical outputs from a given source tree.

Levers we use:

- `Cargo.lock` committed and required (`cargo build --locked`).
- `rust-toolchain.toml` pins the exact compiler version.
- All Rust dependencies pinned to specific versions in `[workspace.dependencies]` (no `^` ranges in production crates' `Cargo.toml`).
- Build flags fixed: `RUSTFLAGS="-C codegen-units=1 -C debuginfo=2"`.
- The Android AAR's `META-INF` and JVM JAR's manifest stripped of timestamps where possible.
- The Swift XCFramework: `xcodebuild -create-xcframework` is deterministic given identical input archives.
- The npm package: `pack` order forced via `files` allowlist and `npm pack --dry-run` diff in CI.

We do not currently produce a publicly-verifiable reproducible build — we produce a *deterministic* one that a contributor could re-run and verify locally. SLSA Level 3 is on the roadmap (Phase 5+).

---

## Release checklist (human-driven, before tagging)

Triggered when a release manager (`@jovachain/sdk-leads`) decides to ship. Checklist:

1. ✅ All open issues with the `release-blocker` label are closed.
2. ✅ `CHANGELOG.md` updated with the new section, including any breaking-change callouts.
3. ✅ All `Cargo.toml` `version = "..."` fields and the `package.json` and `build.gradle.kts` versions match the intended tag.
4. ✅ Latest `main` is green on every CI workflow.
5. ✅ Latest `nightly-fuzz.yml` had no new crashes in the past 7 days.
6. ✅ `cargo-vet` and `cargo-audit` are clean.
7. ✅ Spec vectors validated by `tools/verify-spec`.
8. ✅ A `vX.Y.Z-rc.N` tag has been pushed and `rc_complete` job ran green.
9. ✅ Maven staging artifact smoke-tested in a fresh sandbox app.
10. ✅ npm `@rc` dist-tag artifact smoke-tested in a fresh sandbox web/Node app.
11. ✅ Satellite Swift tag (or a private prerelease branch of it) consumed in a fresh Xcode project; vector tests pass.
12. Tag `vX.Y.Z` on `main`. Push. `publish_release` runs in ordered, least-revertible-last sequence.
13. After publish, smoke-test consumption from each binding's package manager from a clean machine.
14. Watch the GitHub Actions logs through the crates.io step. If anything fails, follow the partial-publish recovery procedure above.

---

## Yanking and post-release fixes

If a critical bug ships:

- **Patch fix** if the bug is fixable without API change → tag `vX.Y.(Z+1)`.
- **Yank** the broken version on crates.io (`cargo yank`) — does not delete, prevents new uses. Maven Central does not allow deletion (only deprecation); we mark the version deprecated. npm allows deprecation (`npm deprecate`).
- **Communication**: GitHub Security Advisory if security-relevant; otherwise CHANGELOG note + Discord/forum announcement.

We do not delete or rewrite history. Yanking and deprecation are the only post-publish operations.

---

## What's *not* in the build pipeline

- **Code signing of the Swift XCFramework with an Apple developer cert.** Not required for SwiftPM consumption; apps re-sign during their own build. We may add this in Phase 5+.
- **Reproducible-builds attestation (SLSA-3).** Roadmap item.
- **Publishing to multiple Maven repositories.** We publish only to Maven Central. Internal mirrors are app-team concerns.
- **Tag-based deployment to staging.** This is an SDK, not a service — there is no "staging environment."
