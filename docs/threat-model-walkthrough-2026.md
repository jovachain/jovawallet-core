# Threat-model walkthrough — 2026

Phase 5 §5 requires walking every "We do not defend against" claim in `docs/security.md` and every audit-checklist item in `docs/memory-and-keys.md`, confirming the implementation actually fits.

This document is the record of that walk. Each section pairs a claim with the implementation evidence (file:line where applicable) and the disposition.

**Status:** Template. The walkthrough runs against `v1.0.0-rc.1` after Phase 5's other tasks ship. Findings update this file in place.

**Sign-off:**
- Engineer A: _TBD_
- Engineer B: _TBD_
- Date: _TBD_
- SDK SHA: _TBD_

---

## "We do not defend against …" claims (from `docs/security.md`)

For each claim, fill in:

- **Evidence:** code path / test that confirms the claim is honored.
- **Status:** ✅ aligned / ❌ misaligned (needs fix) / ⚠️ partially aligned (notes).

### Claim 1: _<paste the exact claim text from security.md>_

**Evidence:** _TBD_  
**Status:** _TBD_

### Claim 2: _<…>_

**Evidence:** _TBD_  
**Status:** _TBD_

_(append one section per claim — `docs/security.md` enumerates them)_

---

## Audit checklist from `docs/memory-and-keys.md`

For each checklist item, fill in:

- **Evidence:** code path / test that proves the item.
- **Status:** ✅ / ❌ / ⚠️.

### Item: `Seed` is `Zeroize + ZeroizeOnDrop`, NOT Clone

**Evidence:** `crates/jova-core-primitives/src/seed.rs` — `#[derive(Zeroize, ZeroizeOnDrop)]`, no `Clone` derive. Verified by `cargo test --workspace`.  
**Status:** _TBD_ (confirm at walkthrough time)

### Item: `XPrv` is `Zeroize + ZeroizeOnDrop`, NOT Clone

**Evidence:** `crates/jova-core-primitives/src/keys.rs`.  
**Status:** _TBD_

### Item: `Ed25519Xprv` is `Zeroize + ZeroizeOnDrop`, NOT Clone

**Evidence:** `crates/jova-core-primitives/src/slip10.rs`.  
**Status:** _TBD_

### Item: `Mnemonic` IS `Clone` (intentional, documented)

**Evidence:** `crates/jova-core-primitives/src/mnemonic.rs` + `docs/memory-and-keys.md` rationale.  
**Status:** _TBD_

### Item: `panic = "abort"` in release profile

**Evidence:** `Cargo.toml [profile.release]`.  
**Status:** _TBD_

### Item: `#![forbid(unsafe_code)]` on every project crate's `lib.rs`

**Evidence:** grep across all project crates' `lib.rs`.  
**Status:** _TBD_

_(append one section per checklist item)_

---

## Misalignments found

(Empty until the walkthrough runs.)

For any ❌ items, file an issue and link here.

---

## Sign-off

After every claim and checklist item is ✅, both engineers sign with date + SHA.
