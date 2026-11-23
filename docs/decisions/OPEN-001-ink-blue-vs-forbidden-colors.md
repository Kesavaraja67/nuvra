# OPEN-001: Ink Blue Pro Theme vs Forbidden Colors

**Status:** OPEN
**Date:** 2026-11-23
**Decider:** Kesavaraja M (Owner)

---

## 1. The Question

Does the locked Pro theme **Ink Blue** (`--surface: #070B14`, `--accent: #7FA3FF` per Sections 6.2 and 15.6) conflict with Rule **R-28** ("Forbidden aesthetics: purple/blue default gradients...") and `docs/BRAND.md` ("Forbidden Brand Elements: Purple or blue as primary/accent color")?

---

## 2. Why It Is Unresolved

- **Rule R-28** forbids "purple/blue default gradients, stock glassmorphism...".
- **`docs/BRAND.md`** lists "Purple or blue as primary/accent color" under Forbidden Brand Elements to prevent generic software defaults.
- **Section 6.2 & 15.6** explicitly specify a locked Pro theme named **Ink Blue** with accent `#7FA3FF` and surface `#070B14`.
- Per the Decision Protocol, rules and locked decisions must be reconciled by the owner.

---

## 3. Options

### Option 1 (Recommended): Scoped Brand Exception for Optional Theme
- **Description:** Clarify that R-28 and the brand color restriction apply to the **default** out-of-the-box appearance (Obsidian theme uses warm orange `#FF6B3D`; Paper theme uses `#E8501F`). The optional, user-selected Pro theme "Ink Blue" is permitted to use `#7FA3FF` as a solid accent (never gradients).
- **Trade-offs:** Maintains the unique warm orange default brand identity while honoring the locked Section 6.2 Pro theme.

### Option 2: Shift Ink Blue Accent to Ice-Teal or Slate
- **Description:** Adjust the Ink Blue accent from `#7FA3FF` to a cyan/ice-teal (`#5CE1E6`) or neutral slate (`#C5D1E0`) that avoids blue/purple hues.
- **Trade-offs:** Avoids blue altogether, but departs from the locked `#7FA3FF` token value.

### Option 3: Replace Ink Blue Theme
- **Description:** Replace the Ink Blue theme with an alternate theme (e.g., "Forest Bronze", `--surface: #0A0F0D`, `--accent: #C2A649`).
- **Trade-offs:** Eliminates any blue from the product, but changes the theme roster in Section 15.6.

---

## 4. Recommendation

**Option 1.** R-28's intent is to prevent Nuvra from looking like generic AI/crypto software that defaults to purple-blue glows and gradients. Having an optional, solid, dark Pro theme named Ink Blue does not dilute the default warm brand identity.
