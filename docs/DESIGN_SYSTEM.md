# Design System — Nuvra
*Source of truth for Section 6 of the master spec. All values are locked. No token may be changed without owner approval via the Decision Protocol.*

---

## Visual Idea

**"Bezel extension."** The notch is a physical continuation of the screen's top bezel: matte surface, flush with the top edge, square top corners, rounded bottom corners, a 1 px inner top-light (`edge-lit`), and one accent used only when something is live. Solid surfaces only. No backdrop blur. No glassmorphism. Album art may tint the surface with a single solid color at ≤ 10% opacity — never a gradient.

---

## 6.2 Color Tokens

### Obsidian (default, dark)
| Token | Value |
|---|---|
| `--surface` | `#060709` |
| `--surface-raised` | `#0E1014` |
| `--surface-high` | `#161920` |
| `--edge` | `#232732` |
| `--edge-lit` | `rgba(255,255,255,0.06)` |
| `--text-primary` | `#F3F4F6` |
| `--text-secondary` | `#9AA0AC` |
| `--text-tertiary` | `#5E6470` |
| `--accent` | `#FF6B3D` |
| `--danger` | `#FF4258` |
| `--ok` | `#46D39A` |
| `--warning` | `#FFC247` |

### Paper (light)
| Token | Value |
|---|---|
| `--surface` | `#F6F5F2` |
| `--surface-raised` | `#FFFFFF` |
| `--surface-high` | `#ECEAE4` |
| `--edge` | `#DAD8D2` |
| `--edge-lit` | `rgba(255,255,255,0.80)` |
| `--text-primary` | `#14151A` |
| `--text-secondary` | `#5A5E68` |
| `--text-tertiary` | `#8A8D96` |
| `--accent` | `#E8501F` |
| `--danger` | `#D42A40` |
| `--ok` | `#1E9E6A` |
| `--warning` | `#B87A00` |

### Signal (high contrast — always free)
| Token | Value |
|---|---|
| `--surface` | `#000000` |
| `--surface-raised` | `#0A0A0A` |
| `--surface-high` | `#141414` |
| `--edge` | `#FFFFFF` |
| `--edge-lit` | `rgba(255,255,255,1.00)` |
| `--text-primary` | `#FFFFFF` |
| `--text-secondary` | `#E0E0E0` |
| `--text-tertiary` | `#BDBDBD` |
| `--accent` | `#FFD400` |
| `--danger` | `#FF6B6B` |
| `--ok` | `#7CFF9E` |
| `--warning` | `#FFD400` |

### Pro Themes (entitlement-gated)
| Theme | `--surface` | `--surface-raised` | `--accent` |
|---|---|---|---|
| Graphite Sand | `#121316` | `#1A1C21` | `#D6C29A` |
| Ink Blue | `#070B14` | `#0D1424` | `#7FA3FF` |

### Contrast Requirements
All text/background pairs must meet WCAG 2.1 AA (4.5:1 body, 3:1 large text). Signal theme must meet 7:1.

---

## 6.3 Typography

- **Font:** Segoe UI Variable (Text/Display axis). No bundled fonts.
- **Figures:** Tabular (`font-variant-numeric: tabular-nums`) for all numbers.
- **Scale (px):**

| Name | Size |
|---|---|
| caption | 11 |
| small | 12 |
| body | 13 |
| title | 15 |
| display | 20 |

---

## 6.3 Spacing Grid

4 px base unit. Allowed values: 4, 8, 12, 16, 24.

---

## 6.3 Radii

| Use | Value |
|---|---|
| Controls | 8 px |
| Notch Ambient | 14 px (bottom corners only) |
| Notch Glance | 16 px (bottom corners only) |
| Notch Expanded | 18 px (bottom corners only) |
| Top corners | 0 px (always square — flush with bezel) |

---

## 6.3 Icons

- **Custom, drawn in-house.** No icon packs.
- 20 px grid, 1.5 px stroke, round caps and joins.
- Delivered as vector paths.

---

## 6.4 Motion

### Spring Parameters
| Transition | Stiffness | Damping | Mass |
|---|---|---|---|
| Expand | 380 | 32 | 1 |
| Collapse | 460 | 40 | 1 |

Pre-baked into DirectComposition piecewise-cubic animation curves. Approximation error ≤ 0.5 px. On interruption: new curve baked from current position and velocity.

### Content Crossfade
120 ms, `cubic-bezier(0.2, 0, 0, 1)`.

### Rules
- All animations are interruptible (reversible mid-flight).
- Reduced motion (system or user toggle): springs replaced by 80 ms fades; visualizer paused.
- Nothing animates for decoration. Animation must communicate a state change.

---

## 6.5 Accessibility Baseline

- UI Automation tree: name, role, state, value for every notch element.
- Full keyboard operation: Tab/Shift+Tab, arrows, Enter, Esc. Focus ring uses `--accent`.
- Screen-reader names for every control.
- Signal (high contrast) honors the Windows system high-contrast setting.
- Reduced motion honored.
- Minimum hit targets: 24×24 px mouse, 44×44 px touch/pen (D-GLB-04).

---

## 6.6 Copy Rules

Short, specific, human. No "Oops". No exclamation marks. No emoji. Say what happened and what the user can do next.

---

## Token Implementation

Tokens are defined in code at `nuvra-render/src/tokens.rs` as typed Rust constants.
CSS custom properties for the settings WebView2 UI are in `settings-ui/src/tokens.css`.

> [!IMPORTANT]
> R-27: Every color, size, radius, duration, and spring in UI code comes from these tokens. No magic numbers. No off-palette colors. Any deviation is a failed delivery.

> [!WARNING]
> R-28 Forbidden aesthetics: purple/blue default gradients, stock glassmorphism, identical rounded cards everywhere, emoji as icons, stock icon packs, lorem ipsum, centered-everything layouts, drop-shadow glows, decorative animation.
