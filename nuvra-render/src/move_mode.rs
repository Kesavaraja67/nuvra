// nuvra-render/src/move_mode.rs
// Move mode state machine with 2D repositioning, magnetic snapping & threshold — Document UI-1 Section 8 & DECIDED-007.
// R-13: No unsafe in this crate.
#![forbid(unsafe_code)]

use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveState {
    Inactive,
    HeldDrag,
    LatchedDrag,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorBounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl MonitorBounds {
    pub fn center_x(&self) -> f32 {
        self.left + (self.right - self.left) / 2.0
    }
}

pub struct MoveModeController {
    state: MoveState,
    initial_offset: MoveOffset,
    current_offset: MoveOffset,
    grab_pointer: (f32, f32),
    r_down_pos: Option<(f32, f32)>,
    is_snapped_x: bool,
    is_snapped_y: bool,
}

impl MoveModeController {
    pub fn new() -> Self {
        Self {
            state: MoveState::Inactive,
            initial_offset: MoveOffset { x: 0.0, y: 0.0 },
            current_offset: MoveOffset { x: 0.0, y: 0.0 },
            grab_pointer: (0.0, 0.0),
            r_down_pos: None,
            is_snapped_x: false,
            is_snapped_y: false,
        }
    }

    pub fn state(&self) -> MoveState {
        self.state
    }

    pub fn is_active(&self) -> bool {
        self.state != MoveState::Inactive
    }

    pub fn current_offset(&self) -> MoveOffset {
        self.current_offset
    }

    pub fn set_offset(&mut self, offset: MoveOffset) {
        self.current_offset = offset;
        self.initial_offset = offset;
    }

    /// On right-button down (records initial down position)
    pub fn on_right_down(&mut self, px: f32, py: f32) {
        if self.state == MoveState::Inactive {
            self.r_down_pos = Some((px, py));
        }
    }

    /// On right-button up
    pub fn on_right_up(&mut self) -> bool {
        self.r_down_pos = None;
        if self.state == MoveState::HeldDrag {
            self.state = MoveState::Inactive;
            true // Dropped
        } else {
            false
        }
    }

    /// On double click / double tap / long press -> LatchedDrag
    pub fn on_double_click(&mut self, px: f32, py: f32) {
        if self.state == MoveState::Inactive {
            self.state = MoveState::LatchedDrag;
            self.initial_offset = self.current_offset;
            self.grab_pointer = (px, py);
        } else {
            // Drop on click in latched mode
            self.state = MoveState::Inactive;
        }
    }

    /// Escape cancels drag and restores initial offset
    pub fn on_escape(&mut self) -> Option<MoveOffset> {
        self.r_down_pos = None;
        if self.is_active() {
            self.state = MoveState::Inactive;
            self.current_offset = self.initial_offset;
            Some(self.initial_offset)
        } else {
            None
        }
    }

    /// Enter drops in place
    pub fn on_enter(&mut self) {
        self.r_down_pos = None;
        self.state = MoveState::Inactive;
    }

    /// Pointer movement with 4px threshold for held drag and 2D magnetic snap
    pub fn on_pointer_move(
        &mut self,
        px: f32,
        py: f32,
        monitor: &MonitorBounds,
        half_width: f32,
        height: f32,
    ) -> (bool, bool) {
        // returns (position_changed, snap_flashed)
        if self.state == MoveState::Inactive {
            if let Some((start_x, start_y)) = self.r_down_pos {
                let dx = px - start_x;
                let dy = py - start_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq >= 16.0 {
                    // 4px threshold squared
                    self.state = MoveState::HeldDrag;
                    self.initial_offset = self.current_offset;
                    self.grab_pointer = (start_x, start_y);
                } else {
                    return (false, false);
                }
            } else {
                return (false, false);
            }
        }

        let delta_x = px - self.grab_pointer.0;
        let delta_y = py - self.grab_pointer.1;

        let mut raw_x = self.initial_offset.x + delta_x;
        let mut raw_y = self.initial_offset.y + delta_y;

        // 1. Magnetic snap to center X (12 px radius)
        let mut snap_flashed = false;
        let mon_center = monitor.center_x();
        let notch_center = mon_center + raw_x;
        if (notch_center - mon_center).abs() <= tokens::timing::SNAP_RADIUS_PX {
            if !self.is_snapped_x {
                snap_flashed = true;
                self.is_snapped_x = true;
            }
            raw_x = 0.0;
        } else {
            self.is_snapped_x = false;
        }

        // 2. Magnetic snap to top edge Y (12 px radius)
        if raw_y.abs() <= tokens::timing::SNAP_RADIUS_PX {
            if !self.is_snapped_y {
                snap_flashed = true;
                self.is_snapped_y = true;
            }
            raw_y = 0.0;
        } else {
            self.is_snapped_y = false;
        }

        // 3. Clamp within monitor bounds (guard against inverted bounds on constrained monitors)
        let min_x = monitor.left + half_width - mon_center;
        let max_x = monitor.right - half_width - mon_center;
        let min_y = 0.0;
        let max_y = (monitor.bottom - monitor.top - height).max(min_y);

        self.current_offset.x = if min_x <= max_x {
            raw_x.clamp(min_x, max_x)
        } else {
            (min_x + max_x) * 0.5
        };
        self.current_offset.y = raw_y.clamp(min_y, max_y);

        (true, snap_flashed)
    }
}

impl Default for MoveModeController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_threshold_and_snap() {
        let mut mc = MoveModeController::new();
        let mon = MonitorBounds {
            left: 0.0,
            top: 0.0,
            right: 1920.0,
            bottom: 1080.0,
        };

        mc.on_right_down(100.0, 100.0);
        // Small 2px move - should NOT activate
        let (changed, _) = mc.on_pointer_move(102.0, 100.0, &mon, 190.0, 38.0);
        assert!(!changed);
        assert_eq!(mc.state(), MoveState::Inactive);

        // 5px move - exceeds 4px threshold -> activates HeldDrag
        let (changed, _) = mc.on_pointer_move(105.0, 100.0, &mon, 190.0, 38.0);
        assert!(changed);
        assert_eq!(mc.state(), MoveState::HeldDrag);

        // Right up drops
        assert!(mc.on_right_up());
        assert_eq!(mc.state(), MoveState::Inactive);
    }

    #[test]
    fn test_2d_move_and_vertical_snap() {
        let mut mc = MoveModeController::new();
        let mon = MonitorBounds {
            left: 0.0,
            top: 0.0,
            right: 1920.0,
            bottom: 1080.0,
        };

        // Enter latched drag via double click
        mc.on_double_click(500.0, 500.0);
        assert_eq!(mc.state(), MoveState::LatchedDrag);

        // Move 2D: dx = +50px, dy = +200px
        let (changed, flashed) = mc.on_pointer_move(550.0, 700.0, &mon, 190.0, 38.0);
        assert!(changed);
        assert!(!flashed);
        assert_eq!(mc.current_offset().x, 50.0);
        assert_eq!(mc.current_offset().y, 200.0);

        // Move near top edge (dy = +8px, within 12px snap radius)
        let (changed, flashed) = mc.on_pointer_move(550.0, 508.0, &mon, 190.0, 38.0);
        assert!(changed);
        assert!(flashed); // Snapped to top edge!
        assert_eq!(mc.current_offset().y, 0.0);

        // Escape restores initial position (0, 0) from active drag
        let restored = mc.on_escape();
        assert_eq!(restored, Some(MoveOffset { x: 0.0, y: 0.0 }));
        assert_eq!(mc.state(), MoveState::Inactive);

        // Separate Inactive case: on_right_down -> on_escape clears r_down_pos without activating drag
        mc.on_right_down(100.0, 100.0);
        assert_eq!(mc.on_escape(), None);
        let (changed, _) = mc.on_pointer_move(120.0, 120.0, &mon, 190.0, 38.0);
        assert!(!changed);
        assert_eq!(mc.state(), MoveState::Inactive);

        mc.on_right_down(100.0, 100.0);
        mc.on_enter();
        let (changed, _) = mc.on_pointer_move(120.0, 120.0, &mon, 190.0, 38.0);
        assert!(!changed);

        // Oversized geometry on a tiny monitor must not panic when clamping
        let tiny = MonitorBounds {
            left: 0.0,
            top: 0.0,
            right: 200.0,
            bottom: 30.0,
        };
        mc.on_double_click(10.0, 10.0);
        let (changed, _) = mc.on_pointer_move(500.0, 500.0, &tiny, 190.0, 50.0);
        assert!(changed);
        assert_eq!(mc.current_offset(), MoveOffset { x: 0.0, y: 0.0 });
    }
}
