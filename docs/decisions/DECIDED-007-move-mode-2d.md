# DECIDED-007: 2D Move Mode (Horizontal and Vertical Repositioning)

**Date:** 2026-11-24  
**Status:** Approved by Product Owner Jacky (Kesavaraja M) at Gate G1  
**Supersedes:** C7 default in UI-1 (horizontal-only restriction)

---

## Decision

The notch repositioning interaction (Move Mode, initiated via right-click with > 4px drag threshold or double-tap/click) shall support **both horizontal and vertical 2D translation** across the screen workspace of any active monitor.

### Requirements:
1. **2D Follow:** When dragging, the notch follows both `X` and `Y` coordinates:
   `new_x = pointer_x - grab_offset_x`, `new_y = pointer_y - grab_offset_y`.
2. **Bounds Clamping:** The entire notch silhouette remains clamped within the monitor bounds (`top >= monitor.top`, `bottom <= monitor.bottom`, `left >= monitor.left`, `right <= monitor.right`).
3. **Magnetic Snapping:**
   - **Top Edge Snap:** When within `SNAP_RADIUS_PX` (12 px) of the top edge (`y = monitor.top`), the notch magnetically docks flush to the screen edge.
   - **Center Snap:** When within 12 px of the monitor's horizontal center, the notch magnetically snaps to center X.
   - Snapping triggers a 1-pixel `--ok` rim flash (`SNAP_FLASH_MS = 80ms`). Moving 12 px past releases the snap.
4. **Persistence:** The saved per-monitor offset records both `offset_x` (from monitor center) and `offset_y` (from monitor top).
5. **Reset:** Settings action "Reset notch position" resets both offsets to (0, 0) (docked top-center).
