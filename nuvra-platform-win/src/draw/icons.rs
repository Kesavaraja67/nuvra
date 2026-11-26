// nuvra-platform-win/src/draw/icons.rs
// Direct2D vector icon rendering from pure path definitions — Document UI-1 Section 7.
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{
    ID2D1RenderTarget, ID2D1SolidColorBrush, D2D1_ELLIPSE, D2D1_ROUNDED_RECT,
};

use super::svg_icons::{draw_bell_svg, draw_wifi_svg};
use nuvra_render::icons::{get_icon, IconKind};

/// Render an icon from pure path definitions positioned at (center_x, center_y) with scale.
pub fn draw_icon(
    rt: &ID2D1RenderTarget,
    kind: IconKind,
    center_x: f32,
    center_y: f32,
    brush: &ID2D1SolidColorBrush,
    scale: f32,
) {
    let def = get_icon(kind);
    let top_left_x = center_x - (def.width * scale) / 2.0;
    let top_left_y = center_y - (def.height * scale) / 2.0;

    // Fast direct primitive rendering for key icons
    unsafe {
        match kind {
            IconKind::Find => {
                // Circle r=5.5 at (8.75, 8.75), line to (16.5, 16.5)
                let ellipse = D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: top_left_x + 8.75 * scale,
                        y: top_left_y + 8.75 * scale,
                    },
                    radiusX: 5.5 * scale,
                    radiusY: 5.5 * scale,
                };
                rt.DrawEllipse(&ellipse, brush, 1.5 * scale, None);
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: top_left_x + 12.9 * scale,
                        y: top_left_y + 12.9 * scale,
                    },
                    D2D_POINT_2F {
                        x: top_left_x + 16.5 * scale,
                        y: top_left_y + 16.5 * scale,
                    },
                    brush,
                    1.5 * scale,
                    None,
                );
            }
            IconKind::Shelf => {
                // Folder body rounded rect + tab
                let rrect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: top_left_x + 2.75 * scale,
                        top: top_left_y + 6.25 * scale,
                        right: top_left_x + 17.25 * scale,
                        bottom: top_left_y + 16.25 * scale,
                    },
                    radiusX: 2.0 * scale,
                    radiusY: 2.0 * scale,
                };
                rt.DrawRoundedRectangle(&rrect, brush, 1.5 * scale, None);
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: top_left_x + 3.0 * scale,
                        y: top_left_y + 6.25 * scale,
                    },
                    D2D_POINT_2F {
                        x: top_left_x + 4.5 * scale,
                        y: top_left_y + 4.0 * scale,
                    },
                    brush,
                    1.5 * scale,
                    None,
                );
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: top_left_x + 4.5 * scale,
                        y: top_left_y + 4.0 * scale,
                    },
                    D2D_POINT_2F {
                        x: top_left_x + 7.5 * scale,
                        y: top_left_y + 4.0 * scale,
                    },
                    brush,
                    1.5 * scale,
                    None,
                );
            }
            IconKind::Clipboard => {
                let rrect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: top_left_x + 4.25 * scale,
                        top: top_left_y + 4.5 * scale,
                        right: top_left_x + 15.75 * scale,
                        bottom: top_left_y + 17.25 * scale,
                    },
                    radiusX: 2.0 * scale,
                    radiusY: 2.0 * scale,
                };
                rt.DrawRoundedRectangle(&rrect, brush, 1.5 * scale, None);
                let clip = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: top_left_x + 7.25 * scale,
                        top: top_left_y + 2.75 * scale,
                        right: top_left_x + 12.75 * scale,
                        bottom: top_left_y + 5.75 * scale,
                    },
                    radiusX: 1.0 * scale,
                    radiusY: 1.0 * scale,
                };
                rt.DrawRoundedRectangle(&clip, brush, 1.5 * scale, None);
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: top_left_x + 7.25 * scale,
                        y: top_left_y + 9.75 * scale,
                    },
                    D2D_POINT_2F {
                        x: top_left_x + 12.75 * scale,
                        y: top_left_y + 9.75 * scale,
                    },
                    brush,
                    1.2 * scale,
                    None,
                );
            }
            IconKind::Mic => {
                let capsule = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: top_left_x + 7.5 * scale,
                        top: top_left_y + 3.0 * scale,
                        right: top_left_x + 12.5 * scale,
                        bottom: top_left_y + 11.0 * scale,
                    },
                    radiusX: 2.5 * scale,
                    radiusY: 2.5 * scale,
                };
                rt.DrawRoundedRectangle(&capsule, brush, 1.5 * scale, None);
                let arc_r = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: top_left_x + 5.25 * scale,
                        top: top_left_y + 7.0 * scale,
                        right: top_left_x + 14.75 * scale,
                        bottom: top_left_y + 14.5 * scale,
                    },
                    radiusX: 4.75 * scale,
                    radiusY: 4.75 * scale,
                };
                rt.DrawRoundedRectangle(&arc_r, brush, 1.5 * scale, None);
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: top_left_x + 10.0 * scale,
                        y: top_left_y + 14.5 * scale,
                    },
                    D2D_POINT_2F {
                        x: top_left_x + 10.0 * scale,
                        y: top_left_y + 17.25 * scale,
                    },
                    brush,
                    1.5 * scale,
                    None,
                );
            }
            IconKind::Bell | IconKind::BellUnread => {
                draw_bell_svg(rt, center_x, center_y, brush, scale);
            }
            IconKind::Gear => {
                // 1. Center hole
                let inner = D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: center_x,
                        y: center_y,
                    },
                    radiusX: 2.2 * scale,
                    radiusY: 2.2 * scale,
                };
                rt.DrawEllipse(&inner, brush, 1.2 * scale, None);

                // 2. Hub circle
                let outer = D2D1_ELLIPSE {
                    point: D2D_POINT_2F {
                        x: center_x,
                        y: center_y,
                    },
                    radiusX: 4.8 * scale,
                    radiusY: 4.8 * scale,
                };
                rt.DrawEllipse(&outer, brush, 1.5 * scale, None);

                // 3. 6 precision gear teeth with rounded tips
                for i in 0..6 {
                    let angle = (i as f32) * (std::f32::consts::PI / 3.0);
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();
                    let p1 = D2D_POINT_2F {
                        x: center_x + cos_a * 4.4 * scale,
                        y: center_y + sin_a * 4.4 * scale,
                    };
                    let p2 = D2D_POINT_2F {
                        x: center_x + cos_a * 7.5 * scale,
                        y: center_y + sin_a * 7.5 * scale,
                    };
                    rt.DrawLine(p1, p2, brush, 2.5 * scale, None);
                }
            }
            IconKind::Wifi => {
                draw_wifi_svg(rt, center_x, center_y, brush, false, scale);
            }
            IconKind::Play => {
                // Solid crisp filled triangle (8px to 17px)
                for step in 0..10 {
                    let t = step as f32 / 9.0;
                    let x = top_left_x + (8.0 + t * 9.0) * scale;
                    let half_h = (1.0 - t) * 6.0 * scale;
                    rt.DrawLine(
                        D2D_POINT_2F {
                            x,
                            y: center_y - half_h,
                        },
                        D2D_POINT_2F {
                            x,
                            y: center_y + half_h,
                        },
                        brush,
                        1.5 * scale,
                        None,
                    );
                }
            }
            IconKind::Pause => {
                let bar1 = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: center_x - 5.5 * scale,
                        top: center_y - 6.0 * scale,
                        right: center_x - 2.0 * scale,
                        bottom: center_y + 6.0 * scale,
                    },
                    radiusX: 1.5 * scale,
                    radiusY: 1.5 * scale,
                };
                let bar2 = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: center_x + 2.0 * scale,
                        top: center_y - 6.0 * scale,
                        right: center_x + 5.5 * scale,
                        bottom: center_y + 6.0 * scale,
                    },
                    radiusX: 1.5 * scale,
                    radiusY: 1.5 * scale,
                };
                rt.FillRoundedRectangle(&bar1, brush);
                rt.FillRoundedRectangle(&bar2, brush);
            }
            IconKind::Previous => {
                // Left vertical bar
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: center_x - 6.0 * scale,
                        y: center_y - 5.5 * scale,
                    },
                    D2D_POINT_2F {
                        x: center_x - 6.0 * scale,
                        y: center_y + 5.5 * scale,
                    },
                    brush,
                    2.0 * scale,
                    None,
                );
                // Left-pointing triangle
                for step in 0..8 {
                    let t = step as f32 / 7.0;
                    let x = center_x + (5.5 - t * 8.0) * scale;
                    let half_h = (1.0 - t) * 5.0 * scale;
                    rt.DrawLine(
                        D2D_POINT_2F {
                            x,
                            y: center_y - half_h,
                        },
                        D2D_POINT_2F {
                            x,
                            y: center_y + half_h,
                        },
                        brush,
                        1.5 * scale,
                        None,
                    );
                }
            }
            IconKind::Next => {
                // Right vertical bar
                rt.DrawLine(
                    D2D_POINT_2F {
                        x: center_x + 6.0 * scale,
                        y: center_y - 5.5 * scale,
                    },
                    D2D_POINT_2F {
                        x: center_x + 6.0 * scale,
                        y: center_y + 5.5 * scale,
                    },
                    brush,
                    2.0 * scale,
                    None,
                );
                // Right-pointing triangle
                for step in 0..8 {
                    let t = step as f32 / 7.0;
                    let x = center_x - (5.5 - t * 8.0) * scale;
                    let half_h = (1.0 - t) * 5.0 * scale;
                    rt.DrawLine(
                        D2D_POINT_2F {
                            x,
                            y: center_y - half_h,
                        },
                        D2D_POINT_2F {
                            x,
                            y: center_y + half_h,
                        },
                        brush,
                        1.5 * scale,
                        None,
                    );
                }
            }
            IconKind::Grip => {
                for col in 0..2 {
                    for row in 0..3 {
                        let dot = D2D1_ELLIPSE {
                            point: D2D_POINT_2F {
                                x: top_left_x + (7.5 + (col as f32) * 5.0) * scale,
                                y: top_left_y + (5.0 + (row as f32) * 5.0) * scale,
                            },
                            radiusX: 1.25 * scale,
                            radiusY: 1.25 * scale,
                        };
                        rt.FillEllipse(&dot, brush);
                    }
                }
            }
            _ => {}
        }
    }
}
