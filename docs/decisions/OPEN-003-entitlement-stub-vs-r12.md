# OPEN-003: Reconciliation of v1.5 Entitlement Check with Rule R-12

**Status:** OPEN
**Date:** 2026-11-23
**Decider:** Kesavaraja M (Owner)

---

## 1. The Question

How should Section 15.6 (*"entitlement stubbed via the license interface until v3 licensing"*) be reconciled with Rule **R-12** (*"No stubs. No todo!(), unimplemented!(), placeholder logic, fake data, commented-out code or dead code in delivered work"* ) prior to v1.5 and v3 implementation?

---

## 2. Why It Is Unresolved

- **Rule R-12** strictly forbids any stubs or placeholder logic in delivered work across all versions.
- **Section 15.6** states: *"Pro themes (Graphite Sand, Ink Blue) included but entitlement-gated (entitlement stubbed via the license interface until v3 licensing)"*.
- **Section 17.7 (v3)** states: *"entitlement gating per D-BIZ-02/03/04 replaces the v1.5 stub"*, meaning the stub would persist across v1.5 and v2. The runtime behavior and interface contract of this stub remain undefined and conflict directly with R-12.

---

## 3. Options

### Option 1 (Recommended): Free Preview in v1.5 (No Stub Logic)
- **Description:** In v1.5, Pro themes are documented as an open "Free Preview" mode without adding fake entitlement check functions. The real cryptographic entitlement check is introduced only in v2 when the server and local license verification are actually implemented.
- **Trade-offs:** 100% compliant with R-12. No stub code or placeholder logic exists at any point.

### Option 2: Document Scoped Exception to R-12 for v1.5 Entitlement Check
- **Description:** Formally record an exception to R-12 authorizing an offline `EntitlementProvider::OfflineDemo` implementation in v1.5.
- **Trade-offs:** Implements Section 15.6 literally, but introduces an exception to an absolute rule.

### Option 3: Defer Pro Gating Entirely to v2
- **Description:** Keep Pro themes in development builds only, and ship them to end users only when v2 with license authentication ships.
- **Trade-offs:** Delays user preview of Pro themes until v2.

---

## 4. Recommendation

**Option 1.** Designating v1.5 Pro themes as a "Free Preview" avoids adding stub functions that violate R-12, while providing users the intended access.
