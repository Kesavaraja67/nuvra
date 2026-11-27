// nuvra-render/src/icons/media.rs
// Custom in-house vector icon path definitions for media transport — Document UI-1 Section 7.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use super::PathCommand;

// ── 13. Previous (18x18 filled): bar x=2..4.5, left triangle (16,3)-(16,15)-(6,9) ──
pub const PREV_COMMANDS: &[PathCommand] = &[
    PathCommand::MoveTo(2.0, 3.0),
    PathCommand::LineTo(4.5, 3.0),
    PathCommand::LineTo(4.5, 15.0),
    PathCommand::LineTo(2.0, 15.0),
    PathCommand::Close,
    PathCommand::MoveTo(16.0, 3.0),
    PathCommand::LineTo(6.0, 9.0),
    PathCommand::LineTo(16.0, 15.0),
    PathCommand::Close,
];

// ── 14. Next (18x18 filled): mirror of Previous ──
pub const NEXT_COMMANDS: &[PathCommand] = &[
    PathCommand::MoveTo(13.5, 3.0),
    PathCommand::LineTo(16.0, 3.0),
    PathCommand::LineTo(16.0, 15.0),
    PathCommand::LineTo(13.5, 15.0),
    PathCommand::Close,
    PathCommand::MoveTo(2.0, 3.0),
    PathCommand::LineTo(12.0, 9.0),
    PathCommand::LineTo(2.0, 15.0),
    PathCommand::Close,
];

// ── 15. Pause (26x26 filled): two bars x=5..11 and x=15..21, y=3..23, r=2 ──
pub const PAUSE_COMMANDS: &[PathCommand] = &[
    PathCommand::MoveTo(7.0, 3.0),
    PathCommand::LineTo(9.0, 3.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 11.0,
        y: 5.0,
    },
    PathCommand::LineTo(11.0, 21.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 9.0,
        y: 23.0,
    },
    PathCommand::LineTo(7.0, 23.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 5.0,
        y: 21.0,
    },
    PathCommand::LineTo(5.0, 5.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 7.0,
        y: 3.0,
    },
    PathCommand::Close,
    PathCommand::MoveTo(17.0, 3.0),
    PathCommand::LineTo(19.0, 3.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 21.0,
        y: 5.0,
    },
    PathCommand::LineTo(21.0, 21.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 19.0,
        y: 23.0,
    },
    PathCommand::LineTo(17.0, 23.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 15.0,
        y: 21.0,
    },
    PathCommand::LineTo(15.0, 5.0),
    PathCommand::ArcTo {
        rx: 2.0,
        ry: 2.0,
        rotation: 0.0,
        large_arc: false,
        sweep: true,
        x: 17.0,
        y: 3.0,
    },
    PathCommand::Close,
];

// ── 16. Play (26x26 filled): triangle (8, 3.5), (22, 13), (8, 22.5), rounded ──
pub const PLAY_COMMANDS: &[PathCommand] = &[
    PathCommand::MoveTo(8.0, 5.5),
    PathCommand::LineTo(8.0, 20.5),
    PathCommand::LineTo(21.0, 13.0),
    PathCommand::Close,
];

// ── 17. Now-playing bars (16x16 filled): 3 bars w=3, gap=2, h=6, 10, 8 ──
pub const BARS_COMMANDS: &[PathCommand] = &[
    PathCommand::MoveTo(2.0, 10.0),
    PathCommand::LineTo(5.0, 10.0),
    PathCommand::LineTo(5.0, 16.0),
    PathCommand::LineTo(2.0, 16.0),
    PathCommand::Close,
    PathCommand::MoveTo(7.0, 6.0),
    PathCommand::LineTo(10.0, 6.0),
    PathCommand::LineTo(10.0, 16.0),
    PathCommand::LineTo(7.0, 16.0),
    PathCommand::Close,
    PathCommand::MoveTo(12.0, 8.0),
    PathCommand::LineTo(15.0, 8.0),
    PathCommand::LineTo(15.0, 16.0),
    PathCommand::LineTo(12.0, 16.0),
    PathCommand::Close,
];
