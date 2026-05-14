# Audit reports

External third-party security audits of `jovawallet-core`. Each audit drops:

- A PDF report at `docs/audits/<auditor>-<year>.pdf` (or `.md` if text-only).
- A response document at `docs/audits/<auditor>-<year>-response.md` listing every finding and its disposition (fixed / accepted risk / out of scope).

## v1.0.0 audit (Phase 5)

**Status:** Not yet engaged. Tracking [issue #5](https://github.com/jovachain/jovawallet-core/issues/5).

**Recommended auditors** (2026 vintage; pick one):
- Trail of Bits — `https://www.trailofbits.com/`
- Cure53 — `https://cure53.de/`
- Halborn — `https://www.halborn.com/`
- NCC Group — `https://www.nccgroup.com/`

**Scope (per `docs/security.md`):**
- `jova-core-primitives` — 8–12 hours
- Each chain (`jova-core-chains::{evm, btc, xrp, sol}`) — 16–24 hours each
- FFI handle lifecycle (`jova-core-ffi`) — 8 hours
- WASM glue (`jova-core-wasm`) — 8 hours
- Build / release pipeline — 4–8 hours

**Total:** ~80–140 hours.

**Materials provided to the auditor:**
- `docs/architecture.md`
- `docs/decisions.md` (the 12 ADRs)
- `docs/memory-and-keys.md`
- `docs/security.md` (threat model + non-defended-against claims)
- The `v0.3.0` tag (Phase 3 complete; cryptographic surface frozen).

**Triage policy:**
- **Critical / High** must be fixed before `v1.0.0` tag.
- **Medium** evaluated; usually fix before `v1.0.0` unless trivially out-of-scope.
- **Low / Informational** documented in the response; fix-when-convenient.

Each fix is its own PR with a vector that demonstrates the bug.

## Past audits

(None yet. This section appends after each audit completes.)
