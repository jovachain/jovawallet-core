# Phase 5: Hardening + Audit + RC + v1.0 (Process Plan)

> **Status:** Process plan. Mostly procedure (audit coordination, RC cycles, checklist execution) rather than code changes. The agent's role is mostly to run scripts, gather evidence, and respond to audit findings with code patches.

> **For agentic workers:** Each task here is concrete and self-contained, but they're checklist items rather than failing-test/passing-test cycles. Mark each off as evidence is gathered.

**Goal:** Ship `v1.0.0` — the API contract is locked from this point. External audit complete; findings remediated; full reproducible-build verification; bug bounty announced.

**Preconditions:**
- Phase 4 complete: both apps at 100% rollout for at least one full release cycle.
- No known release-blocker bugs.
- `cargo audit`, `cargo deny`, `cargo vet` clean.
- 14 consecutive days of nightly fuzz with no new crashes (Phase 5's first deliverable extends Phase 0's `nightly-fuzz.yml` cadence).

**Exit criteria:**
- External audit completed; high/critical findings fixed; medium/low documented in `docs/audits/<auditor>-2026.md`.
- `v1.0.0-rc.1` (and any subsequent RCs) staged successfully.
- `v1.0.0` tagged on `main`. All artifacts (Rust crates, SwiftPM satellite repo, Maven Central, GitHub release) published in least-revertible-last order. Crates.io has the v1.0.0 publish.
- `CHANGELOG.md` documents every notable change `v0.0.1 → v1.0.0`.
- `spec/api.md` frozen as the v1.0 reference. Future minor versions append.
- Bug bounty program opened publicly with the v1.0 announcement.

---

## Tasks

### 1. Fuzz hardening (week 1)

- [ ] Run `nightly-fuzz.yml` for 14 consecutive days. Goal: no new crashes.
- [ ] If a fuzzer crashes: file an issue, add the input as a regression vector to `spec/test-vectors.json`, fix the underlying parser, restart the 14-day clock.
- [ ] At the end of week 1, capture the corpus state in `jovachain/jovawallet-core-fuzz-corpus`.
- [ ] Run `cargo fuzz coverage` and confirm key parsers (`PSBT decode`, `EIP-1559 RLP decode`, `EIP-712 typed data`, `Solana v0 message`, `XRP canonical`) hit ≥80% line coverage.

### 2. Property-test depth bump + mutation testing (2 days)

- [ ] Increase `proptest` cases per property to 4096 in CI for nightly job.
- [ ] If any property flakes, fix or relax the property — flakiness is a smell.
- [ ] Document any properties that are too expensive at 4096 cases and run them at 256 cases on PRs but 4096 nightly.
- [ ] Add **`cargo-mutants`** to the nightly job. Run mutation testing on `jova-core-primitives` and `jova-core-chains`. Goal: every "killed" mutant is good, surviving mutants indicate weak test coverage. Investigate each surviving mutant; either add a test that catches it or document why the mutation is semantically equivalent.
- [ ] Add **`cargo-machete`** to the audit pipeline. Catches unused workspace dependencies — common rot in long-lived projects.

### 3. miri pass on the full primitives layer (2 days)

- [ ] `cargo +nightly miri test -p jova-core-primitives` clean.
- [ ] Add miri to `nightly-miri.yml` if not already (Phase 0 has it scheduled; verify it's actually been running).
- [ ] If miri reports UB, fix before any RC.

### 4. Reproducible-build verification (2 days)

- [ ] Two engineers, each on a separate machine, build from the same Git SHA.
- [ ] Compare SHA-256 of every artifact:
  ```bash
  shasum -a 256 target/release/libjova_core_ffi.a
  shasum -a 256 bindings/swift/JovaCoreFFI.xcframework/*.dylib
  shasum -a 256 bindings/wasm/pkg/jova_core_wasm_bg.wasm
  shasum -a 256 bindings/kotlin/jova-core/build/outputs/aar/*.aar
  ```
- [ ] All match → reproducibility confirmed. All mismatch → investigate; usually it's a non-deterministic build flag (timestamps, codegen-units, randomness in dep code generation). Fix and retry.
- [ ] Document the SHA-256s of the canonical v1.0.0 build in `docs/release-checksums.md`.

### 5. Threat-model gap audit (3 days)

- [ ] Walk every "We do not defend against" claim in `docs/security.md` and confirm the implementation actually fits.
- [ ] Walk every audit checklist item in `docs/memory-and-keys.md` and confirm.
- [ ] If any claim is misaligned with implementation, fix one or fix the other.
- [ ] Record the audit walk-through as `docs/threat-model-walkthrough-2026.md`.

### 6. External paid audit (4–6 weeks; runs in parallel with the rest of Phase 5)

- [ ] Pick an auditor specializing in cryptographic libraries (Trail of Bits, Cure53, Halborn, NCC Group are all reasonable fits in 2026).
- [ ] Provide the auditor with: `docs/architecture.md`, `docs/decisions.md`, `docs/memory-and-keys.md`, `docs/security.md`, the v0.5.0 tag.
- [ ] Recommended scope (matches `docs/security.md`): primitives 8–12h, each chain 16–24h, FFI handle lifecycle 8h, FFI/wasm glue 8h, build/release 4–8h. Total ~80–140 hours.
- [ ] Findings triage:
  - **Critical / High:** must be fixed before v1.0 tag.
  - **Medium:** evaluate; usually fix before v1.0 unless trivially out-of-scope.
  - **Low / Informational:** document in the response report; fix-when-convenient.
- [ ] Publish the audit report (PDF) at `docs/audits/<auditor>-2026.pdf` (or .md if text-only).
- [ ] Each fix is its own PR with a vector that demonstrates the bug.

### 7. Adopt `release-plz` for automated releases (1 day)

The Phase 0 release-workflow runs on tag-push and is good enough through v0.5.x. For v1.0+ the team should adopt **`release-plz`**:

- Automatic semver bumps based on Conventional Commits.
- Auto-generated CHANGELOG entries.
- PRs that update `Cargo.toml` versions and CHANGELOG, ready to merge.
- Eliminates the human "remember to bump versions before tagging" step.

`release-plz` integrates cleanly with the existing tag-triggered publish workflow — it produces the tags, the existing pipeline does the multi-registry publish in the documented order.

Set it up under `.github/workflows/release-plz.yml`. The release manager reviews PRs as usual; merging triggers the existing release pipeline.

### 8. Pre-RC release-pipeline dry-run (3 days)

- [ ] Tag a `v1.0.0-rc.1` on a branch (not `main` until ready).
- [ ] Run the full release workflow. It must:
  - Run preflight checks.
  - Build XCFramework / AAR / WASM / Rust crates artifacts.
  - Stage to Maven Central OSSRH (close staging repo, do not release).
  - Publish to npm with `--tag rc`.
  - Stop at `rc_complete` (does not push satellite Swift, does not release Maven, does not publish to crates.io).
- [ ] Smoke-test the staged Maven artifact via `tools/release/smoke-test-maven-staging.sh`.
- [ ] Smoke-test the npm RC dist-tag via `npm install @jovachain/wallet-core@rc` in a fresh sandbox.
- [ ] If any step fails: drop the staged Maven repo, fix, tag `rc.2`, repeat.

### 9. App-team RC validation (1 week)

- [ ] iOS app builds against `v1.0.0-rc.1` SDK in a feature branch. Run the full app test suite.
- [ ] Android app builds against `1.0.0-rc.1` (npm RC also tested by web team if they exist yet).
- [ ] Both teams confirm: behavior identical to the production `0.5.x` line.
- [ ] If a regression is found: SDK fix, `rc.2`, repeat.

### 10. v1.0 tag, full publish (1 day)

- [ ] After RC clean and app validation green: tag `v1.0.0` on `main`.
- [ ] Push the tag. Release workflow runs in non-RC mode and publishes in least-revertible-last order:
  1. Swift satellite repo push.
  2. Maven Central release-from-staging.
  3. npm dist-tag promote `rc → latest`.
  4. crates.io publish (`jova-core-primitives`, `jova-core-chains`, `jova-core`).
  5. GitHub release with checksums.
- [ ] Watch the workflow logs through the crates.io step. Any failure: follow `docs/build-and-release.md` partial-publish recovery.
- [ ] Post-publish: smoke-test consumption from each registry on a fresh machine.

### 11. Bug bounty program launch (in parallel with task 10)

- [ ] Pick a platform: Immunefi (large-blast crypto programs) or HackerOne (broader audience).
- [ ] Define scope: `jova-core-primitives`, `jova-core-chains`, `jova-core`, `jova-core-ffi`, `jova-core-wasm` at the v1.0.0 tag.
- [ ] Out of scope (document explicitly):
  - Issues only reproducible in debug builds.
  - Issues requiring root / jailbreak.
  - Issues in upstream crates (report to the crate maintainer).
  - Issues in app-side code.
- [ ] Severity rewards (suggested): Critical $25k–$100k, High $5k–$25k, Medium $1k–$5k, Low $250.
- [ ] Funding committed before launch.
- [ ] Announcement coordinated with the v1.0.0 release post.

### 12. Post-publish verification (1 day)

- [ ] Fresh sandbox app on Linux: `cargo add jova-core` resolves; sample hello-world signs.
- [ ] Fresh Xcode project on macOS: SwiftPM resolves `jovawallet-core-swift@1.0.0`; sample hello-world signs.
- [ ] Fresh Android Studio project: Maven Central resolves `io.jovachain:jova-core:1.0.0`; sample hello-world signs.
- [ ] Fresh `pnpm create vite` project: `pnpm add @jovachain/wallet-core` resolves; sample hello-world signs.
- [ ] All four sandboxes load the same vector and produce byte-identical output.

### 13. Documentation freeze (0.5 day)

- [ ] Copy `docs/api.md` → `spec/api.md`. Frozen forever; future minor versions append.
- [ ] Copy `docs/chains.md` → `spec/chains.md`. Same.
- [ ] Copy the relevant section of `docs/error-model.md` → `spec/errors.md`. Same.
- [ ] Run `cargo run -p jova-verify-spec` to confirm no drift.
- [ ] Open `spec/CHANGELOG.md`'s `[1.0]` entry. Future spec changes append here, never modify history.

### 14. CHANGELOG and release notes (1 day)

- [ ] Write the `[1.0.0]` section of `CHANGELOG.md` summarizing every notable change since `v0.0.1`.
- [ ] Write the GitHub release notes — public-facing, app-developer-friendly.
- [ ] Coordinate the announcement post with marketing if applicable.

---

## What this plan produces

**Tag:** `v1.0.0` on `main`.
**Artifacts:** crates.io publish, satellite Swift repo at v1.0.0, Maven Central `1.0.0`, npm `@jovachain/wallet-core@1.0.0`, GitHub release with all binaries + checksums.
**Documents:** `docs/audits/<auditor>-2026.{md,pdf}`, `docs/release-checksums.md`, `docs/threat-model-walkthrough-2026.md`, frozen `spec/` snapshot.
**Programs:** Bug bounty live.

## What this plan does NOT do

- Does not implement new chains. (Phase 8 if a custom Jova chain ships post-v1.0.)
- Does not change the public API. The whole point of the v1.0 tag is *locking* the API.
- Does not handle WASM functional rollout. Phase 6 ships separately at v1.1.0.

## Estimated time

3–4 weeks of SDK-team work. The audit runs in parallel and is the longest pole.
