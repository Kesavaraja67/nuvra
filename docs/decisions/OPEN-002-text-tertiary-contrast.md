# OPEN-002: Text-Tertiary Contrast Ratio in Obsidian Theme

**Status:** OPEN
**Date:** 2026-11-23
**Decider:** Kesavaraja M (Owner)

---

## 1. The Question

Does the locked color token `--text-tertiary: #5E6470` on Obsidian surface `#060709` (~3.5:1 contrast) satisfy WCAG 2.1 AA accessibility requirements given the Section 6.2 rule: *"All text/background pairs must meet WCAG 2.1 AA (4.5:1 body, 3:1 large text)"*?

---

## 2. Why It Is Unresolved

- On Obsidian surface (`#060709`), `#5E6470` yields a contrast ratio of **3.5:1**.
- WCAG 2.1 AA requires **4.5:1** for normal body text, and **3.0:1** for large text (>= 18pt or 14pt bold), incidental text, or non-text UI components.
- If `--text-tertiary` is used for body copy, it fails 4.5:1. If restricted to non-body/incidental elements, it passes 3.0:1.

---

## 3. Options

### Option 1 (Recommended): Restrict `--text-tertiary` to Non-Text & Incidental Indicators
- **Description:** Formally specify in the design system that `--text-tertiary` is reserved strictly for non-text UI components, borders, subtle divider lines, and disabled/incidental indicators where 3.0:1 is the WCAG AA requirement (or exempt). All text copy—including normal-size input placeholder text—must use `--text-primary` (16.2:1) or `--text-secondary` (`#9AA0AC`, 7.4:1) to satisfy the 4.5:1 body text requirement.
- **Trade-offs:** Preserves the locked `#5E6470` palette token while ensuring every text element meets WCAG 2.1 AA (4.5:1 for body and placeholders, 3:1 for large text and UI components).

### Option 2: Lighten `--text-tertiary` in Obsidian to 4.5:1
- **Description:** Adjust Obsidian `--text-tertiary` from `#5E6470` to `#757B88` (which provides 4.6:1 contrast against `#060709`).
- **Trade-offs:** Every text token meets 4.5:1 unconditionally, but reduces the visual hierarchy contrast between secondary and tertiary text.

### Option 3: Eliminate `--text-tertiary`
- **Description:** Consolidate secondary and tertiary text tokens into a single `--text-secondary` (`#9AA0AC`, 7.4:1).
- **Trade-offs:** Simplifies the token system, but reduces typographic nuance in dense UI surfaces.

---

## 4. Recommendation

**Option 1.** In modern design systems (including Windows Fluent and Apple HIG), tertiary text is by definition incidental metadata or timestamps where 3:1 applies. Explicitly documenting this usage rule keeps the locked tokens intact and accessible.
