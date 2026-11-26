// nuvra-render/src/icons/mod.rs
// Custom in-house vector icon path definitions — Document UI-1 Section 7.
// R-13: No unsafe in this crate.
// R-28: No emoji as icons, no stock packs. All defined on 20x20 unit grid (or designated grid).
#![forbid(unsafe_code)]

pub mod media;
pub mod system;

use media::*;
use system::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    ArcTo {
        rx: f32,
        ry: f32,
        rotation: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Find,
    Shelf,
    Clipboard,
    Mic,
    MicMuted,
    Bell,
    BellUnread,
    Gear,
    Wifi,
    Battery,
    Close,
    Clear,
    Grip,
    Previous,
    Next,
    Play,
    Pause,
    NowPlayingBars,
    LogoN,
}

#[derive(Debug, Clone)]
pub struct IconDef {
    pub kind: IconKind,
    pub width: f32,
    pub height: f32,
    pub stroke_width: f32,
    pub is_filled: bool,
    pub commands: &'static [PathCommand],
}

pub fn get_icon(kind: IconKind) -> IconDef {
    match kind {
        IconKind::Find => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: FIND_COMMANDS,
        },
        IconKind::Shelf => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: SHELF_COMMANDS,
        },
        IconKind::Clipboard => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: CLIPBOARD_COMMANDS,
        },
        IconKind::Mic => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: MIC_COMMANDS,
        },
        IconKind::MicMuted => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: MIC_MUTED_COMMANDS,
        },
        IconKind::Bell => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: BELL_COMMANDS,
        },
        IconKind::BellUnread => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: BELL_COMMANDS,
        },
        IconKind::Gear => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: GEAR_COMMANDS,
        },
        IconKind::Wifi => IconDef {
            kind,
            width: 18.0,
            height: 18.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: WIFI_COMMANDS,
        },
        IconKind::Battery => IconDef {
            kind,
            width: 24.0,
            height: 12.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: BATTERY_COMMANDS,
        },
        IconKind::Close => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: CLOSE_COMMANDS,
        },
        IconKind::Clear => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: false,
            commands: CLEAR_COMMANDS,
        },
        IconKind::Grip => IconDef {
            kind,
            width: 20.0,
            height: 20.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: GRIP_COMMANDS,
        },
        IconKind::Previous => IconDef {
            kind,
            width: 18.0,
            height: 18.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: PREV_COMMANDS,
        },
        IconKind::Next => IconDef {
            kind,
            width: 18.0,
            height: 18.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: NEXT_COMMANDS,
        },
        IconKind::Play => IconDef {
            kind,
            width: 26.0,
            height: 26.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: PLAY_COMMANDS,
        },
        IconKind::Pause => IconDef {
            kind,
            width: 26.0,
            height: 26.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: PAUSE_COMMANDS,
        },
        IconKind::NowPlayingBars => IconDef {
            kind,
            width: 16.0,
            height: 16.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: BARS_COMMANDS,
        },
        IconKind::LogoN => IconDef {
            kind,
            width: 22.0,
            height: 22.0,
            stroke_width: 1.5,
            is_filled: true,
            commands: LOGO_N_COMMANDS,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_icons_have_commands() {
        let kinds = [
            IconKind::Find,
            IconKind::Shelf,
            IconKind::Clipboard,
            IconKind::Mic,
            IconKind::MicMuted,
            IconKind::Bell,
            IconKind::BellUnread,
            IconKind::Gear,
            IconKind::Wifi,
            IconKind::Battery,
            IconKind::Close,
            IconKind::Clear,
            IconKind::Grip,
            IconKind::Previous,
            IconKind::Next,
            IconKind::Play,
            IconKind::Pause,
            IconKind::NowPlayingBars,
            IconKind::LogoN,
        ];

        for kind in kinds {
            let icon = get_icon(kind);
            assert!(
                !icon.commands.is_empty(),
                "Icon {:?} has empty commands",
                kind
            );
            assert!(icon.width > 0.0);
            assert!(icon.height > 0.0);
        }
    }
}
