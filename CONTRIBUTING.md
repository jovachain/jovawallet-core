# Contributing to jovawallet-core

Thanks for your interest. See `docs/README.md` for project context.

## Process
1. Open an issue describing the change before opening a PR (skip for trivial fixes).
2. Branch from `main`. Use descriptive names: `feat/btc-bip322`, `fix/eip712-domain`.
3. Every PR must be green on every CI workflow.
4. Every behavior change must be reflected in `spec/test-vectors.json`.
5. PRs touching public API require an ADR addition in `docs/decisions.md`.

## Commit messages
Conventional Commits format: `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`. Multi-line bodies welcome for non-trivial changes.

## Test vectors
A new chain or behavior is not "supported" without vectors. See `docs/testing.md` for the new-chain checklist.

## Security
Vulnerability disclosure: see `SECURITY.md`.
