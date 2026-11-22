# Nuvra — Brand System

> Source of truth for brand identity: wordmark, logomark, app icon, and brand tone.
> All assets must be created originally (R-26: no copies from other apps or stock packs).
> Applied by: `brandkit` skill (Phase 0).

---

## Brand Idea

**Bezel extension.** Nuvra feels like hardware, not software. It is the top of the screen made useful — quiet when nothing is happening, specific when it needs your attention, then gone. The brand speaks in the same register: minimal, unhyped, precise.

---

## Wordmark

**Typeface:** Geist Mono (OFL-1.1). Kerning: −20. Weight: Regular.
**Case:** All lowercase — `nuvra`.
**Color:** `--text-primary` from active theme. Never a gradient. Never outlined.
**Usage:** Settings window header, about screen, marketing site. Not on the notch itself.

> Geist Mono source: [https://github.com/vercel/geist-font](https://github.com/vercel/geist-font) — OFL-1.1 — listed in THIRD_PARTY_LICENSES.md.

### Clearspace
Minimum clearspace = 1× cap-height on all sides.

### Forbidden uses
- Do not render in any color other than `--text-primary` or `--accent`.
- Do not alter letter-spacing beyond the locked value.
- Do not combine with a tagline in the same visual unit as the wordmark.

---

## Logomark

**Concept:** The letter **N** constructed from two vertical strokes bridged by a horizontal gap — the gap is the notch slot. The negative space reads as a cut through the top edge of a screen. No filled shapes; the form is built entirely from strokes.

**Grid:** 24 × 24 px design grid. 1.5 px stroke weight. Round caps.
**Color:** Monochrome — matches `--text-primary`. One variant uses `--accent` for the bridge/gap element only.

### SVG specification (owner to implement or commission)

```
Artboard: 24 × 24 px
Stroke: 1.5 px, round linecap, round linejoin
Left vertical:   M 5 20 L 5 4
Right vertical:  M 19 4 L 19 20
Bridge (diagonal): M 5 4 L 19 20
Gap marker: 1 px gap centered at the bridge midpoint (12,12) — no fill, no stroke; it is simply absent
```

> This is a vector description for the owner or designer to execute. The file `assets/logomark.svg` must be created by the owner. It is listed as OWNER_FILL in the handoff.

---

## App Icon

**Form:** The 24 × 24 logomark centered on a `--surface` (#060709) fill in Obsidian theme.
**Sizes required:** 16, 32, 48, 64, 128, 256, 512 px. Delivered as `.ico` (all sizes multiplexed) and individual `.png` files.
**Corner radius:** 20% of canvas dimension per Windows 11 icon spec.
**Foreground:** `--text-primary` (#F3F4F6) on Obsidian. No shadow. No gradient. No gloss.

> File path: `assets/icon.ico`, `assets/icon-{size}.png`. Listed as OWNER_FILL in handoff.

---

## Brand Tone

| Dimension | Value |
|---|---|
| Formal ↔ Casual | Slightly casual — no jargon, no filler |
| Serious ↔ Playful | Serious — utility first |
| Enthusiastic ↔ Matter-of-fact | Matter-of-fact |
| Complex ↔ Sparse | Sparse — say less |

**Voice rules:**
- Short sentences. Noun + verb. Active voice.
- No exclamation marks. No "Oops". No emoji in copy.
- Numbers are specific: "last 20 items", not "recent items".
- Error messages say what happened and what the user can do next. Nothing else.

---

## Forbidden Brand Elements

Applies to every surface (notch, settings, marketing site, Android app):

- Purple or blue as primary/accent color.
- Any gradient in the brand palette.
- Glassmorphism / backdrop blur as a brand element.
- Rounded cards as the dominant layout pattern.
- Drop-shadow glows.
- Stock icon packs.
- Emoji used as icons.
- Taglines on the same line as the wordmark.
- Uppercase wordmark.

---

## Asset Checklist

| Asset | File | Status |
|---|---|---|
| Wordmark (SVG) | `assets/wordmark.svg` | OWNER_FILL |
| Logomark (SVG) | `assets/logomark.svg` | OWNER_FILL |
| App icon (.ico) | `assets/icon.ico` | OWNER_FILL |
| App icon 16–512 (.png) | `assets/icon-{n}.png` | OWNER_FILL |
| Android launcher icon | `android/app/src/main/res/mipmap-*/` | OWNER_FILL |
| OG image (marketing) | `marketing/og.png` | OWNER_FILL (v3 scope) |

---

## Third-Party License Note

Geist Mono is licensed OFL-1.1. It is listed in `THIRD_PARTY_LICENSES.md`. The font is used for the wordmark rendering only; it is not bundled in the app binary (Segoe UI Variable is the app UI font — a Windows system font, no bundling required).
