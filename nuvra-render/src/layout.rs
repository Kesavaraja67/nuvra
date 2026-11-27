// nuvra-render/src/layout.rs
// Pure layout engine — Document UI-1 Sections 6 & 10.3.
// R-13: No unsafe. R-27: Tokens only. U-07: Pure layout function returning physical rects.
#![forbid(unsafe_code)]

use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutState {
    Collapsed,
    Expanded,
    Banner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemId {
    Logo,
    StatusZone,
    Wifi,
    Battery,
    Clock,
    AlbumArt,
    TrackText,
    Prev,
    PlayPause,
    Next,
    DividerA,
    Find,
    Shelf,
    Clipboard,
    Mic,
    DividerB,
    Bell,
    Badge,
    DividerC,
    Settings,
    BannerAppIcon,
    BannerText,
    BannerClose,
    DrawerContainer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
}

#[derive(Debug, Clone)]
pub struct LayoutItem {
    pub id: ItemId,
    pub visual_rect: Rect,
    pub hit_rect: Rect,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub state: LayoutState,
    pub scale: f32,
    pub body: Rect,
    pub silhouette: Rect,
    pub items: Vec<LayoutItem>,
}

/// Snap logical edge to physical pixel boundary (Section 6.9 rule 1).
#[inline]
fn snap_edge(logical: f32, scale: f32) -> f32 {
    (logical * scale).round()
}

/// Computes snapped physical rect from logical coordinates.
fn snap_rect(left: f32, top: f32, right: f32, bottom: f32, scale: f32) -> Rect {
    Rect {
        left: snap_edge(left, scale),
        top: snap_edge(top, scale),
        right: snap_edge(right, scale),
        bottom: snap_edge(bottom, scale),
    }
}

/// Pure layout function converting UI-1 specification into snapped physical pixel layout.
pub fn compute_layout(
    state: LayoutState,
    scale: f32,
    measured_clock_width: f32,
    media_present: bool,
    unread_count: u32,
    drawer_height: Option<f32>,
) -> Layout {
    let mut items = Vec::with_capacity(24);

    let (logical_w, mut logical_h) = match state {
        LayoutState::Collapsed => (
            tokens::window::COLLAPSED_WIDTH,
            tokens::window::COLLAPSED_HEIGHT,
        ),
        LayoutState::Expanded => (
            tokens::window::EXPANDED_WIDTH,
            tokens::window::EXPANDED_HEIGHT,
        ),
        LayoutState::Banner => (tokens::window::BANNER_WIDTH, tokens::window::BANNER_HEIGHT),
    };

    if let Some(dh) = drawer_height {
        let clamped_dh = dh.clamp(
            tokens::window::DRAWER_MIN_HEIGHT,
            tokens::window::DRAWER_MAX_HEIGHT,
        );
        logical_h += clamped_dh;
    }

    // Force physical body width to even integer (Section 6.9 rule 2)
    let body_left_px = 0.0;
    let mut body_right_px = snap_edge(logical_w, scale);
    if ((body_right_px - body_left_px) as i32) % 2 != 0 {
        body_right_px += 1.0;
    }
    let body_bottom_px = snap_edge(logical_h, scale);
    let body = Rect::new(body_left_px, 0.0, body_right_px, body_bottom_px);

    // Silhouette includes the two 10px concave shoulder wings at top (Section 6.2)
    let fillet_px = snap_edge(tokens::radius::FILLET, scale);
    let silhouette = Rect::new(
        body_left_px - fillet_px,
        0.0,
        body_right_px + fillet_px,
        body_bottom_px,
    );

    let hit_h_44 = snap_edge(44.0, scale);

    match state {
        LayoutState::Collapsed => {
            // Collapsed: Logo (x 22), Status zone (x 84..176), Clock (x 211..360)
            let cy = tokens::window::COLLAPSED_HEIGHT / 2.0;

            // 1. Logo (22x22, centered at y=19)
            let logo_visual = snap_rect(22.0, cy - 11.0, 44.0, cy + 11.0, scale);
            let logo_hit = snap_rect(10.0, 0.0, 50.0, tokens::window::COLLAPSED_HEIGHT, scale);
            items.push(LayoutItem {
                id: ItemId::Logo,
                visual_rect: logo_visual,
                hit_rect: logo_hit,
            });

            // 2. Status zone (84 to 176): Wi-Fi (18x18), Battery (24x12)
            let wifi_visual = snap_rect(84.0, cy - 9.0, 102.0, cy + 9.0, scale);
            let wifi_hit = snap_rect(80.0, 0.0, 110.0, tokens::window::COLLAPSED_HEIGHT, scale);
            items.push(LayoutItem {
                id: ItemId::Wifi,
                visual_rect: wifi_visual,
                hit_rect: wifi_hit,
            });

            let bat_visual = snap_rect(116.0, cy - 6.0, 180.0, cy + 6.0, scale);
            let bat_hit = snap_rect(112.0, 0.0, 185.0, tokens::window::COLLAPSED_HEIGHT, scale);
            items.push(LayoutItem {
                id: ItemId::Battery,
                visual_rect: bat_visual,
                hit_rect: bat_hit,
            });

            // 3. Date + Time cluster (right aligned with 20px padding)
            let clock_reserved_w = 149.0f32.max(measured_clock_width);
            let clock_start_x =
                (tokens::window::COLLAPSED_WIDTH - 20.0 - clock_reserved_w).max(200.0);
            let clock_visual = snap_rect(
                clock_start_x,
                cy - 10.0,
                tokens::window::COLLAPSED_WIDTH - 20.0,
                cy + 10.0,
                scale,
            );
            let clock_hit = snap_rect(
                clock_start_x - 4.0,
                0.0,
                tokens::window::COLLAPSED_WIDTH,
                tokens::window::COLLAPSED_HEIGHT,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Clock,
                visual_rect: clock_visual,
                hit_rect: clock_hit,
            });
        }

        LayoutState::Expanded => {
            let cy = tokens::window::EXPANDED_TOOLBAR_HEIGHT / 2.0; // 24.0

            if media_present {
                // 1. Album art (14 to 48, 34x34 centered in 48px horizontal bar)
                let art_v = snap_rect(14.0, cy - 17.0, 48.0, cy + 17.0, scale);
                let art_h = snap_rect(12.0, 3.0, 50.0, 45.0, scale);
                items.push(LayoutItem {
                    id: ItemId::AlbumArt,
                    visual_rect: art_v,
                    hit_rect: art_h,
                });

                // Flex text block calculation
                let clock_w = measured_clock_width.clamp(140.0, 160.0);
                let text_w = (145.0 - (clock_w - 149.0)).clamp(120.0, 160.0);
                let text_right = 60.0 + text_w;

                let txt_v = snap_rect(60.0, cy - 16.0, text_right, cy + 16.0, scale);
                let txt_h = snap_rect(58.0, 3.0, text_right, 45.0, scale);
                items.push(LayoutItem {
                    id: ItemId::TrackText,
                    visual_rect: txt_v,
                    hit_rect: txt_h,
                });

                // Transport controls: Previous, Play/Pause, Next
                let prev_v = snap_rect(225.0 + 9.0, cy - 9.0, 225.0 + 27.0, cy + 9.0, scale);
                let prev_h = snap_rect(
                    225.0,
                    cy - hit_h_44 / (2.0 * scale),
                    261.0,
                    cy + hit_h_44 / (2.0 * scale),
                    scale,
                );
                items.push(LayoutItem {
                    id: ItemId::Prev,
                    visual_rect: prev_v,
                    hit_rect: prev_h,
                });

                let play_v = snap_rect(269.0 + 9.0, cy - 13.0, 269.0 + 35.0, cy + 13.0, scale);
                let play_h = snap_rect(
                    269.0,
                    cy - hit_h_44 / (2.0 * scale),
                    313.0,
                    cy + hit_h_44 / (2.0 * scale),
                    scale,
                );
                items.push(LayoutItem {
                    id: ItemId::PlayPause,
                    visual_rect: play_v,
                    hit_rect: play_h,
                });

                let next_v = snap_rect(321.0 + 9.0, cy - 9.0, 321.0 + 27.0, cy + 9.0, scale);
                let next_h = snap_rect(
                    321.0,
                    cy - hit_h_44 / (2.0 * scale),
                    357.0,
                    cy + hit_h_44 / (2.0 * scale),
                    scale,
                );
                items.push(LayoutItem {
                    id: ItemId::Next,
                    visual_rect: next_v,
                    hit_rect: next_h,
                });

                // Divider A (371, 1x24 centered on cy=24)
                let div_a_v = snap_rect(371.0, 12.0, 372.0, 36.0, scale);
                items.push(LayoutItem {
                    id: ItemId::DividerA,
                    visual_rect: div_a_v,
                    hit_rect: div_a_v,
                });
            } else {
                // No media session: Logo at x=20 (22x22) and "Nothing playing" (Section 6.5)
                let logo_v = snap_rect(20.0, cy - 11.0, 42.0, cy + 11.0, scale);
                let logo_h = snap_rect(16.0, 6.0, 46.0, 50.0, scale);
                items.push(LayoutItem {
                    id: ItemId::Logo,
                    visual_rect: logo_v,
                    hit_rect: logo_h,
                });

                let text_v = snap_rect(54.0, cy - 10.0, 190.0, cy + 10.0, scale);
                let text_h = snap_rect(50.0, 6.0, 194.0, 50.0, scale);
                items.push(LayoutItem {
                    id: ItemId::TrackText,
                    visual_rect: text_v,
                    hit_rect: text_h,
                });
            }

            // Items 7 to 15 remain anchored with full toolbar width
            let x_offset = 0.0;

            // 7. Find (386 to 422)
            let find_v = snap_rect(
                386.0 + x_offset,
                cy - 18.0,
                422.0 + x_offset,
                cy + 18.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Find,
                visual_rect: find_v,
                hit_rect: find_v,
            });

            // 8. Shelf (430 to 466)
            let shelf_v = snap_rect(
                430.0 + x_offset,
                cy - 18.0,
                466.0 + x_offset,
                cy + 18.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Shelf,
                visual_rect: shelf_v,
                hit_rect: shelf_v,
            });

            // 9. Clipboard (474 to 510)
            let clip_v = snap_rect(
                474.0 + x_offset,
                cy - 18.0,
                510.0 + x_offset,
                cy + 18.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Clipboard,
                visual_rect: clip_v,
                hit_rect: clip_v,
            });

            // 10. Mic (518 to 554)
            let mic_v = snap_rect(
                518.0 + x_offset,
                cy - 18.0,
                554.0 + x_offset,
                cy + 18.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Mic,
                visual_rect: mic_v,
                hit_rect: mic_v,
            });

            // 11. Divider B (568, 1x24 centered on cy=24)
            let div_b_v = snap_rect(568.0 + x_offset, 12.0, 569.0 + x_offset, 36.0, scale);
            items.push(LayoutItem {
                id: ItemId::DividerB,
                visual_rect: div_b_v,
                hit_rect: div_b_v,
            });

            // 12. Bell (587 to 619, 32x32)
            let bell_v = snap_rect(
                587.0 + x_offset,
                cy - 16.0,
                619.0 + x_offset,
                cy + 16.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Bell,
                visual_rect: bell_v,
                hit_rect: bell_v,
            });

            // Badge (14x14 circle neatly tucked on upper-right of bell)
            if unread_count > 0 {
                let badge_v = snap_rect(
                    616.0 + x_offset - 7.0,
                    11.0 - 7.0,
                    616.0 + x_offset + 7.0,
                    11.0 + 7.0,
                    scale,
                );
                items.push(LayoutItem {
                    id: ItemId::Badge,
                    visual_rect: badge_v,
                    hit_rect: badge_v,
                });
            }

            // 13. Divider C (637, 1x24)
            let div_c_v = snap_rect(637.0 + x_offset, 12.0, 638.0 + x_offset, 36.0, scale);
            items.push(LayoutItem {
                id: ItemId::DividerC,
                visual_rect: div_c_v,
                hit_rect: div_c_v,
            });

            // 14. Clock (652 to 802, reserved 150)
            let clock_v = snap_rect(
                652.0 + x_offset,
                cy - 10.0,
                802.0 + x_offset,
                cy + 10.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Clock,
                visual_rect: clock_v,
                hit_rect: clock_v,
            });

            // 15. Settings gear (816 to 848, 32x32 plate)
            let gear_v = snap_rect(
                816.0 + x_offset,
                cy - 16.0,
                848.0 + x_offset,
                cy + 16.0,
                scale,
            );
            items.push(LayoutItem {
                id: ItemId::Settings,
                visual_rect: gear_v,
                hit_rect: gear_v,
            });

            // Drawer container (if open)
            if drawer_height.is_some() {
                let drawer_v = snap_rect(16.0, 56.0, logical_w - 16.0, logical_h - 16.0, scale);
                items.push(LayoutItem {
                    id: ItemId::DrawerContainer,
                    visual_rect: drawer_v,
                    hit_rect: drawer_v,
                });
            }
        }

        LayoutState::Banner => {
            let cy = tokens::window::BANNER_HEIGHT / 2.0;
            // App icon 32x32 at x=12
            let icon_v = snap_rect(12.0, cy - 16.0, 44.0, cy + 16.0, scale);
            items.push(LayoutItem {
                id: ItemId::BannerAppIcon,
                visual_rect: icon_v,
                hit_rect: icon_v,
            });

            // Title + Body text block (56 to 440)
            let text_v = snap_rect(56.0, 8.0, 440.0, 48.0, scale);
            items.push(LayoutItem {
                id: ItemId::BannerText,
                visual_rect: text_v,
                hit_rect: text_v,
            });

            // Close button (visual 448..468, hit 442..474 >= 24x24)
            let close_v = snap_rect(448.0, cy - 10.0, 468.0, cy + 10.0, scale);
            let close_h = snap_rect(442.0, cy - 14.0, 474.0, cy + 14.0, scale);
            items.push(LayoutItem {
                id: ItemId::BannerClose,
                visual_rect: close_v,
                hit_rect: close_h,
            });
        }
    }

    Layout {
        state,
        scale,
        body,
        silhouette,
        items,
    }
}
