// nuvra-render/src/choreo.rs
// Choreography timelines — Document UI-1 Sections 9.2, 9.4, 9.5, 9.6.
// Pure data structures defining animation transitions and layer timings.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    Expand,
    Collapse,
    OpenDrawer,
    CloseDrawer,
    EdgeWake,
    EdgeWakeReverse,
}

#[derive(Debug, Clone)]
pub struct LayerTiming {
    pub start_ms: f32,
    pub duration_ms: f32,
    pub from_opacity: f32,
    pub to_opacity: f32,
    pub rise_px: f32,
}

#[derive(Debug, Clone)]
pub struct TransitionChoreography {
    pub kind: TransitionKind,
    pub total_duration_ms: f32,
    pub shell_from_width: f32,
    pub shell_to_width: f32,
    pub shell_from_height: f32,
    pub shell_to_height: f32,
    pub collapsed_content: LayerTiming,
    pub toolbar_center_groups: LayerTiming,
    pub toolbar_outer_groups: LayerTiming,
    pub drawer_layer: Option<LayerTiming>,
    pub edge_wake_layer: Option<LayerTiming>,
}

/// Computes the exact expand choreography per Section 9.2.
pub fn get_expand_choreo(with_media: bool, reduced_motion: bool) -> TransitionChoreography {
    let to_w = if with_media {
        tokens::window::EXPANDED_WIDTH
    } else {
        tokens::window::EXPANDED_NO_MEDIA_WIDTH
    };

    if reduced_motion {
        return TransitionChoreography {
            kind: TransitionKind::Expand,
            total_duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
            shell_from_width: tokens::window::COLLAPSED_WIDTH,
            shell_to_width: to_w,
            shell_from_height: tokens::window::COLLAPSED_HEIGHT,
            shell_to_height: tokens::window::EXPANDED_HEIGHT,
            collapsed_content: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 1.0,
                to_opacity: 0.0,
                rise_px: 0.0,
            },
            toolbar_center_groups: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 0.0,
                to_opacity: 1.0,
                rise_px: 0.0,
            },
            toolbar_outer_groups: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 0.0,
                to_opacity: 1.0,
                rise_px: 0.0,
            },
            drawer_layer: None,
            edge_wake_layer: None,
        };
    }

    TransitionChoreography {
        kind: TransitionKind::Expand,
        total_duration_ms: 305.0, // Spring settles within 1% at 305ms
        shell_from_width: tokens::window::COLLAPSED_WIDTH,
        shell_to_width: to_w,
        shell_from_height: tokens::window::COLLAPSED_HEIGHT,
        shell_to_height: tokens::window::EXPANDED_HEIGHT,
        // Collapsed content fades out 0..70ms
        collapsed_content: LayerTiming {
            start_ms: 0.0,
            duration_ms: tokens::timing::COLLAPSED_FADE_OUT_MS,
            from_opacity: 1.0,
            to_opacity: 0.0,
            rise_px: 0.0,
        },
        // Center groups (tools, bell) fade in 60..200ms with 4px rise
        toolbar_center_groups: LayerTiming {
            start_ms: tokens::timing::TOOLBAR_STAGGER_CENTER_MS,
            duration_ms: tokens::timing::TOOLBAR_FADE_IN_MS,
            from_opacity: 0.0,
            to_opacity: 1.0,
            rise_px: tokens::timing::TOOLBAR_RISE_PX,
        },
        // Outer groups (media, clock, gear) fade in 100..240ms with 4px rise
        toolbar_outer_groups: LayerTiming {
            start_ms: tokens::timing::TOOLBAR_STAGGER_OUTER_MS,
            duration_ms: tokens::timing::TOOLBAR_FADE_IN_MS,
            from_opacity: 0.0,
            to_opacity: 1.0,
            rise_px: tokens::timing::TOOLBAR_RISE_PX,
        },
        drawer_layer: None,
        edge_wake_layer: None,
    }
}

/// Computes the exact collapse choreography per Section 9.4.
pub fn get_collapse_choreo(
    from_w: f32,
    from_h: f32,
    reduced_motion: bool,
) -> TransitionChoreography {
    if reduced_motion {
        return TransitionChoreography {
            kind: TransitionKind::Collapse,
            total_duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
            shell_from_width: from_w,
            shell_to_width: tokens::window::COLLAPSED_WIDTH,
            shell_from_height: from_h,
            shell_to_height: tokens::window::COLLAPSED_HEIGHT,
            collapsed_content: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 0.0,
                to_opacity: 1.0,
                rise_px: 0.0,
            },
            toolbar_center_groups: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 1.0,
                to_opacity: 0.0,
                rise_px: 0.0,
            },
            toolbar_outer_groups: LayerTiming {
                start_ms: 0.0,
                duration_ms: tokens::timing::REDUCED_MOTION_FADE_MS,
                from_opacity: 1.0,
                to_opacity: 0.0,
                rise_px: 0.0,
            },
            drawer_layer: None,
            edge_wake_layer: None,
        };
    }

    TransitionChoreography {
        kind: TransitionKind::Collapse,
        total_duration_ms: 262.0, // Collapse spring settles at 262ms
        shell_from_width: from_w,
        shell_to_width: tokens::window::COLLAPSED_WIDTH,
        shell_from_height: from_h,
        shell_to_height: tokens::window::COLLAPSED_HEIGHT,
        // Collapsed content fades in starting at 120ms
        collapsed_content: LayerTiming {
            start_ms: tokens::timing::COLLAPSED_FADE_IN_DELAY_MS,
            duration_ms: tokens::timing::COLLAPSED_FADE_IN_MS,
            from_opacity: 0.0,
            to_opacity: 1.0,
            rise_px: 0.0,
        },
        // Toolbar content fades out 0..80ms
        toolbar_center_groups: LayerTiming {
            start_ms: 0.0,
            duration_ms: tokens::timing::TOOLBAR_FADE_OUT_MS,
            from_opacity: 1.0,
            to_opacity: 0.0,
            rise_px: 0.0,
        },
        toolbar_outer_groups: LayerTiming {
            start_ms: 0.0,
            duration_ms: tokens::timing::TOOLBAR_FADE_OUT_MS,
            from_opacity: 1.0,
            to_opacity: 0.0,
            rise_px: 0.0,
        },
        drawer_layer: None,
        edge_wake_layer: None,
    }
}
