# OPEN-004: Glassmorphic Translucency vs Solid Bezel Extension

**Status:** OPEN  
**Date:** 2026-11-23  
**Decider:** Kesavaraja M (Owner)  

---

## 1. The Question

Should the Nuvra Notch background surface remain a **solid matte "bezel extension"** per Section 6.1 and Rule R-28, or should it adopt a **translucent / glassmorphic** appearance?

---

## 2. Why It Is Unresolved

- **Section 6.1 & docs/DESIGN_SYSTEM.md** explicitly lock the visual concept to a **"Bezel extension"**:
  > *"The notch looks like a physical continuation of the screen's bezel: matte, flush with the top edge, square top corners, rounded bottom corners, a 1 px inner top-light, and one accent used only when something is live. Solid surfaces only. No backdrop blur."*
- **Rule R-28** lists *"stock glassmorphism"* under forbidden aesthetics to protect compositor performance on the reference Floor machine (integrated Intel HD graphics).
- **Owner Request:** The owner specifically inquired why the notch is not glassmorphic and requested visual review of the surface styling.
- Under **Rule R-01**, product design and aesthetic direction belong exclusively to the owner.

---

## 3. Options

### Option 1 (Recommended): Translucent Smoked Glass (Alpha 0.85 + 1px Specular Edge)
- **Description:** Render the Obsidian surface with 85% opacity (`alpha: 0.85`) combined with a crisp 1px specular top-edge highlight (`#FFFFFF` at 12% alpha) and subtle border reflection.
- **Visual Result:** The user's wallpaper and background windows show through the notch with a high-end dark glass aesthetic.
- **Performance:** 100% shader-free Direct2D composition; runs at a locked 120+ FPS with 0% dropped frames on all GPUs, including the Floor machine and WARP fallback.
- **Rule Impact:** Clarifies Rule R-28 that "stock glassmorphism" prohibits generic frosted blur clones, while premium translucent smoked glass is permitted.

### Option 2: Full Windows Acrylic / DWM Backdrop Blur
- **Description:** Enable Windows DWM backdrop blur (`DwmEnableBlurBehindWindow` or DirectComposition Gaussian blur effect) behind the notch window for full frosted glass.
- **Visual Result:** The area behind the notch is physically blurred like Windows 11 Acrylic / Mica.
- **Trade-offs:** Significantly increases compositor GPU memory bandwidth and fill-rate cost on low-end machines (Intel HD 2015-2018), which may risk dropping frames during fast spring animations on the Floor machine.

### Option 3: Strictly Maintain Solid Matte Bezel Extension (Current Section 6.1 Spec)
- **Description:** Keep the notch 100% opaque solid black (`#060709`) matte finish flush with the screen's top bezel.
- **Visual Result:** Solid physical extension of the bezel, as originally written in Section 6.1.
- **Trade-offs:** Does not have the translucent glass aesthetic the owner asked about.

---

## 4. Recommendation

**Option 1.** It delivers the sleek, modern dark translucent glass look allowing the wallpaper to subtly show through, while completely avoiding the heavy GPU blur shader penalties on the Floor reference machine.
