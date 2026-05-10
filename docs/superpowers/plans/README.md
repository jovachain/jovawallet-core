# Implementation Plans — index

Per-phase plans for delivering `jovawallet-core` from empty repo to v1.2.0. Each plan covers one phase and ends in a tagged release or a milestone document.

For project-level orientation, read `/CLAUDE.md` at the repo root first. This file is the execution-roadmap detail.

## The plans

| # | Plan | Tag at end | Type |
|---|---|---|---|
| -1 | [Feasibility spike](./2026-05-05-phase-minus-1-feasibility-spike.md) | (no tag) | Full TDD |
| 0 | [Repo bootstrap](./2026-05-05-phase-0-repo-bootstrap.md) | `v0.0.1` | Full TDD |
| 1 | [EVM end-to-end](./2026-05-05-phase-1-evm-end-to-end.md) | `v0.1.0` | Full TDD |
| 2 | [Bitcoin](./2026-05-05-phase-2-bitcoin.md) | `v0.2.0` | Full TDD |
| 3 | [Solana + XRP + remaining EVM](./2026-05-05-phase-3-sol-xrp-remaining-evm.md) | `v0.5.0` | Full TDD |
| 4 | [iOS + Android app integration](./2026-05-05-phase-4-app-integration.md) | (no SDK tag) | Process |
| 5 | [Hardening + audit + RC + v1.0](./2026-05-05-phase-5-hardening-audit-v1.md) | `v1.0.0` | Process |
| 6 | [WASM functional + npm](./2026-05-05-phase-6-wasm-functional.md) | `v1.1.0` | Full TDD |
| 7 | [Hardware-wallet readiness](./2026-05-05-phase-7-hardware-readiness.md) | `v1.2.0` | Process |

(Phase 8 — custom Jova chain — is not pre-planned. When the chain ships, the work is a single one-day chore: add the `JovaChain` variant, add three vectors, tag a minor version. See `docs/plan.md` for the contingency.)

### Plan types

- **Full TDD** — step-level tasks with exact file paths, code, and commands. Several phases (1, 2, 3, 6) include a vector-capture step that runs an external reference signer (`cast`, `bdk-cli`, `solana-cli`, `rippled`/`xrpl-py`); captured values populate `spec/test-vectors.json`. `tools/verify-spec` rejects placeholder strings, so committed vectors are always real.
- **Process** — checklist-driven phases where work is mostly procedure (audit coordination, app-repo rollout, RC cycles). Each task is a checklist item, not a failing-test/passing-test cycle.

## Dependency graph

```
-1 (feasibility) ──► 0 (bootstrap) ──► 1 (EVM) ──► 2 (BTC) ──► 3 (SOL/XRP/etc)
                                                                     │
                                                                     ▼
                                                           4 (app integration, in app repos)
                                                                     │
                                                                     ▼
                                                           5 (hardening + v1.0)
                                                                     │
                                                       ┌─────────────┼─────────────┐
                                                       ▼             ▼             ▼
                                                  6 (WASM)    7 (hardware)    8 (Jova chain — when ready)
```

Phase 6 and Phase 7 can run in parallel after Phase 5; both depend on `v1.0.0`.
Phase 4 must complete before Phase 5 — the audit reflects code exercised by real apps in production.

## Execution recipe

For each plan, in order:

1. **Read the plan's "Preconditions" section.** Confirm everything is satisfied. If a tool is missing, install it before starting.
2. **Branch from `main`** (or from the prior phase's merge commit). Phase -1 uses `spike/feasibility`, which is throwaway — Phase 0 starts from a clean `main`.
3. **Dispatch the plan** using `superpowers:subagent-driven-development` (recommended): fresh subagent per task, two-stage review per task (spec compliance → code quality), fix loops until clean.
4. **Each task's commit step is one commit.** Don't squash mid-task.
5. **End of phase: open a PR** titled `Phase N: <summary>`. CI must be green on every workflow.
6. **Tag at the merge commit** if the plan calls for one. Phase -1 doesn't tag.
7. **Verify exit criteria** before starting the next plan. If gaps remain, address with patch releases (`v0.X.Y+1`) before proceeding.

## Time budget

| Phase | Honest range for a senior team |
|---|---|
| -1 | 3–5 days |
| 0 | 3–5 days |
| 1 | 10–14 days |
| 2 | 3–4 weeks |
| 3 | 3–5 weeks |
| 4 | 3–4 weeks (parallel iOS + Android) |
| 5 | 3–4 weeks (audit runs in parallel) |
| **Sub-total to v1.0** | **~16–22 weeks (4–5.5 months)** |
| 6 | 2–3 weeks |
| 7 | 2 weeks of SDK-side work + indeterminate firmware-side work |

Add 50–100% buffer for less-experienced teams or for any phase where the prior phase exposed unknowns.

## Start here

**Execute Phase -1 first.** It's a 3–5 day toolchain-validation spike that produces `docs/feasibility-report.md`.

Stop after Phase -1. The user reads the report and decides go/no-go for Phase 0.

Do not skip Phase -1. Its purpose is to discover toolchain incompatibilities before writing real code. A no-go from the spike redirects Phase 0 (e.g., swap a chain crate); a go means Phase 0 is straightforward.

## When in doubt

- A plan's task seems to conflict with `docs/architecture.md`, `docs/api.md`, `docs/decisions.md`, or `docs/security.md` → trust the docs, ask the user.
- A vector's expected value is uncertain → capture from a reference signer; never write a value by hand.
- A code snippet in a plan doesn't compile against the current crate version → trust the test, adjust the snippet (test-as-contract).
- `cargo miri`, `cargo audit`, `cargo deny`, or `cargo vet` flag something → stop, fix, then continue. Don't bypass.
- A CI workflow is red on a PR → read the failure log; fix in a new commit; do not skip the workflow.
- You feel uncertain whether your approach is correct → stop. Use `BLOCKED` or `NEEDS_CONTEXT`. Bad work is worse than no work.

## Where commits land

All commits land in the `jovawallet-core` repo (this one). The satellite Swift repo `jovachain/jovawallet-core-swift` is populated by CI on every release, never by the agent directly.
