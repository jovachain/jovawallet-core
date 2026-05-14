# Phase 5 status — hardening + audit + v1.0.0

Tracking the path from `v0.3.0` to `v1.0.0`. Phase 5 is process-heavy; most blockers are external (audit firm, app-team RC validation, bug bounty funding). This document captures what's in-repo and what's pending.

## In-repo scaffolding (shipped from this Linux dev VM)

| Deliverable | Path | Status |
|---|---|---|
| Nightly proptest @ 4096 cases | `.github/workflows/nightly-hardening.yml` | ✅ Wired |
| `cargo-mutants` mutation testing | `.github/workflows/nightly-hardening.yml` | ✅ Wired (reports surfaced as workflow artifact; human review gates v1.0) |
| `cargo-machete` (unused-dep check) | `.github/workflows/nightly-hardening.yml` | ✅ Wired |
| `release-plz` automated semver | `.github/workflows/release-plz.yml`, `release-plz.toml`, `release-plz.changelog.toml` | ✅ Scaffolded (`workflow_dispatch` until v1.0; flips to push-to-main after) |
| Audit report directory | `docs/audits/README.md` | ✅ Scaffolded |
| Reproducible-build checksums template | `docs/release-checksums.md` | ✅ Scaffolded; fills in at v1.0.0-rc.1 |
| Threat-model walkthrough template | `docs/threat-model-walkthrough-2026.md` | ✅ Scaffolded; fills in at v1.0.0-rc.1 |

## Open release gates (tracked as GitHub issues)

The `v1.0.0` tag is blocked until each of these closes:

| # | Gate | Tracking | Owner |
|---|---|---|---|
| 1 | 14 consecutive days of nightly fuzz with no new crashes | [#14](https://github.com/jovachain/jovawallet-core/issues/14) | SDK team |
| 2 | External audit complete; high/critical findings fixed | [#8](https://github.com/jovachain/jovawallet-core/issues/8) | SDK team + auditor |
| 3 | Reproducible-build dual-engineer pairing | [#9](https://github.com/jovachain/jovawallet-core/issues/9) | SDK team |
| 4 | Threat-model walkthrough — every `docs/security.md` "we do not defend against" claim aligned with implementation | [#10](https://github.com/jovachain/jovawallet-core/issues/10) | SDK team |
| 5 | App-team RC validation: iOS + Android apps build cleanly against `v1.0.0-rc.1` SDK and pass app test suites | [#11](https://github.com/jovachain/jovawallet-core/issues/11) | app teams |
| 6 | Bug bounty program funding committed; scope + reward tiers documented | [#12](https://github.com/jovachain/jovawallet-core/issues/12) | engineering management |
| 7 | Phase 4 100%-rollout soak: both apps on every chain for one full release cycle | [#13](https://github.com/jovachain/jovawallet-core/issues/13) | app teams |
| 8 | Phase 2 BTC migration spot-check (already tracked) | [#3](https://github.com/jovachain/jovawallet-core/issues/3) | Android team |
| 9 | Phase 2 BTC mainnet smoke (already tracked) | [#4](https://github.com/jovachain/jovawallet-core/issues/4) | engineer-driven |

## RC validation sequence

When every gate is closed:

1. Tag `v1.0.0-rc.1` on a release branch (not `main` until ready).
2. Release pipeline runs in `--rc` mode: builds artifacts, stages Maven Central OSSRH (close staging — do not release), publishes npm with `--tag rc`. **Does not** push the Swift satellite, **does not** release Maven, **does not** publish to crates.io.
3. App teams pin `1.0.0-rc.1` in a feature branch; run full app test suites.
4. Smoke-test the staged Maven artifact + the npm RC dist-tag in clean sandboxes.
5. If anything regresses: drop the staged Maven repo, fix, tag `rc.2`, repeat.
6. Once all RCs are green and apps have validated: tag `v1.0.0` on `main`. Release pipeline runs in non-RC mode and publishes in least-revertible-last order:
   - Swift satellite repo push.
   - Maven Central release-from-staging.
   - npm dist-tag promote `rc` → `latest`.
   - crates.io publish (`jova-core-primitives`, `jova-core-chains`, `jova-core`).
   - GitHub release with checksums.

## Post-publish verification

Run from clean sandboxes on each platform:

- Linux: `cargo add jova-core` resolves; sample hello-world signs.
- macOS: Xcode SwiftPM resolves `jovawallet-core-swift@1.0.0`; sample hello-world signs.
- Android Studio: Maven Central resolves `io.jovachain:jova-core:1.0.0`; sample hello-world signs.
- `pnpm create vite` + `pnpm add @jovachain/wallet-core`: resolves; sample hello-world signs.

All four sandboxes load the same vector and produce byte-identical output.

## Bug bounty (parallel with the v1.0 tag)

Platform candidates: Immunefi (crypto-focused), HackerOne (broader).

Scope: `jova-core-primitives`, `jova-core-chains`, `jova-core`, `jova-core-ffi`, `jova-core-wasm` at the `v1.0.0` tag.

Out of scope (documented at launch):
- Issues only reproducible in debug builds.
- Issues requiring root / jailbreak.
- Issues in upstream crates (report to maintainer).
- Issues in app-side code.

Suggested rewards (funding committed before launch):
- Critical: $25k–$100k
- High: $5k–$25k
- Medium: $1k–$5k
- Low: $250

## What Phase 5 does NOT do

- Does not implement new chains.
- Does not change the public API (v1.0 locks it).
- Does not handle WASM functional rollout (Phase 6 → `v1.1.0`).
