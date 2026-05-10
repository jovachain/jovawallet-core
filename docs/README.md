# jovawallet-core — Internal Docs

Repo-internal documentation. Every load-bearing decision behind `jovawallet-core` lives here. If you're contributing to it, integrating with it, auditing it, or trying to understand why it looks the way it does — start here.

## Reading order

Pick the entry that matches what you're doing.

- **First time on this repo.** `overview.md` → `architecture.md` → `decisions.md` → `api.md`.
- **Adding a chain.** `chains.md` → `flows.md` → `testing.md`.
- **Integrating from an iOS app.** `integration-ios.md` → `api.md`.
- **Integrating from an Android app.** `integration-android.md` → `api.md`.
- **Picking up implementation work.** `plan.md` → `folder-structure.md` → `flows.md`.
- **Auditing.** `security.md` → `memory-and-keys.md` → `architecture.md` → `testing.md`.
- **Bringing the SDK to a new platform.** `integration-web.md` / `integration-backend.md` / `integration-hardware.md`.

## Index

### Foundations
| File | What it covers |
|---|---|
| [`overview.md`](./overview.md) | What `jovawallet-core` is, what it isn't, who consumes it |
| [`architecture.md`](./architecture.md) | Rust core + multi-binding architecture; layered crates |
| [`decisions.md`](./decisions.md) | ADRs for every load-bearing choice |
| [`folder-structure.md`](./folder-structure.md) | File-by-file repo layout |
| [`glossary.md`](./glossary.md) | Terms used across the docs |

### Contracts
| File | What it covers |
|---|---|
| [`api.md`](./api.md) | Public `JovaWallet` API: types, methods, semantics |
| [`chains.md`](./chains.md) | Per-chain registry: derivation, address format, tx shape |
| [`error-model.md`](./error-model.md) | `JovaError` taxonomy and per-language mapping |
| [`flows.md`](./flows.md) | Sequence diagrams for every public operation |

### Engineering
| File | What it covers |
|---|---|
| [`memory-and-keys.md`](./memory-and-keys.md) | Secure memory handling, zeroization, FFI key clearing |
| [`testing.md`](./testing.md) | Vectors, property tests, fuzzing, parity strategy |
| [`build-and-release.md`](./build-and-release.md) | CI matrix, publishing, semver lockstep |
| [`security.md`](./security.md) | Threat model, audit posture, supply-chain controls |
| [`plan.md`](./plan.md) | Phased build plan from empty repo to v1.0 |

### Integration guides
| File | What it covers |
|---|---|
| [`integration-ios.md`](./integration-ios.md) | iOS / SwiftPM consumption |
| [`integration-android.md`](./integration-android.md) | Android / Maven / AAR consumption |
| [`integration-web.md`](./integration-web.md) | Browser / Node / WASM consumption (Phase 6) |
| [`integration-backend.md`](./integration-backend.md) | Rust / Node backend consumption (Phase 6) |
| [`integration-hardware.md`](./integration-hardware.md) | Hardware-wallet firmware consumption (Phase 7) |

## Status

Pre-implementation. The architecture, API contract, and chain registry described here are **target shapes** — not yet code. Phase 0 of [`plan.md`](./plan.md) cuts the actual repo and lands the first compilable Rust workspace.

When in doubt, the source-of-truth ordering is:

1. `spec/test-vectors.json` — if a vector says X, the SDK must produce X.
2. `spec/api.md` (a copy of `docs/api.md` lives there at v1.0) — if behavior differs, this contract wins.
3. `docs/decisions.md` — if a code review challenges an architectural choice, the ADR explains why.
4. Everything else here is supporting context.
