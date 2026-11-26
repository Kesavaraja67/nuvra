// nuvra-platform-win/src/draw/text.rs
// DirectWrite text formatting and brush helpers — Document UI-1 Sections 6.8 & 10.6.
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::core::w;
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{
    ID2D1RenderTarget, ID2D1SolidColorBrush, D2D1_DRAW_TEXT_OPTIONS_CLIP,
};
use windows::Win32::Graphics::DirectWrite::{
    IDWriteFactory, IDWriteInlineObject, IDWriteTextFormat, DWRITE_FONT_STRETCH_NORMAL,
    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_MEDIUM,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TRIMMING,
    DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_WORD_WRAPPING_NO_WRAP,
};

use super::{DrawBrushes, TextFormats};
use nuvra_render::engine::ColorScheme;

pub fn color_from_u32(c: u32) -> D2D1_COLOR_F {
    let a = ((c >> 24) & 0xFF) as f32 / 255.0;
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    D2D1_COLOR_F { r, g, b, a }
}

pub fn update_brushes(brushes: &DrawBrushes, colors: &ColorScheme) {
    unsafe {
        brushes.surface.SetColor(&color_from_u32(colors.surface));
        brushes
            .surface_raised
            .SetColor(&color_from_u32(colors.surface_raised));
        brushes
            .surface_high
            .SetColor(&color_from_u32(colors.surface_high));
        brushes.edge.SetColor(&color_from_u32(colors.edge));
        brushes
            .plate_rest
            .SetColor(&color_from_u32(colors.plate_rest));
        brushes
            .plate_hover
            .SetColor(&color_from_u32(colors.plate_hover));
        brushes
            .plate_active
            .SetColor(&color_from_u32(colors.plate_active));
        brushes.divider.SetColor(&color_from_u32(colors.divider));
        brushes
            .text_primary
            .SetColor(&color_from_u32(colors.text_primary));
        brushes
            .text_secondary
            .SetColor(&color_from_u32(colors.text_secondary));
        brushes
            .text_tertiary
            .SetColor(&color_from_u32(colors.text_tertiary));
        brushes.accent.SetColor(&color_from_u32(colors.accent));
        brushes
            .accent_hi
            .SetColor(&color_from_u32(colors.accent_hi));
        brushes.danger.SetColor(&color_from_u32(colors.danger));
        brushes.ok.SetColor(&color_from_u32(colors.ok));
        brushes.warning.SetColor(&color_from_u32(colors.warning));
        brushes.logo_a.SetColor(&color_from_u32(colors.logo_a));
        brushes.logo_b.SetColor(&color_from_u32(colors.logo_b));
    }
}

unsafe fn make_fmt(
    factory: &IDWriteFactory,
    weight: windows::Win32::Graphics::DirectWrite::DWRITE_FONT_WEIGHT,
    size: f32,
    align: windows::Win32::Graphics::DirectWrite::DWRITE_TEXT_ALIGNMENT,
    no_wrap: bool,
) -> Result<IDWriteTextFormat, String> {
    let fmt = factory
        .CreateTextFormat(w!("Segoe UI Variable Text"), None, weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, size, w!("en-us"))
        .or_else(|_| factory.CreateTextFormat(w!("Segoe UI"), None, weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, size, w!("en-us")))
        .map_err(|e| format!("TextFormat failed: {e}"))?;
    let _ = fmt.SetTextAlignment(align);
    let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
    if no_wrap {
        let _ = fmt.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
        let trim = DWRITE_TRIMMING {
            granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER,
            delimiter: 0,
            delimiterCount: 0,
        };
        if let Ok(ellipsis) = factory.CreateEllipsisTrimmingSign(&fmt) {
            let _ = fmt.SetTrimming(&trim, &ellipsis);
        } else {
            let _ = fmt.SetTrimming(&trim, None::<&IDWriteInlineObject>);
        }
    }
    Ok(fmt)
}

pub fn create_text_formats(factory: &IDWriteFactory) -> Result<TextFormats, String> {
    // SAFETY: Valid DirectWrite factory creating immutable text format handles.
    unsafe {
        Ok(TextFormats {
            display: make_fmt(factory, DWRITE_FONT_WEIGHT_SEMI_BOLD, 20.0, DWRITE_TEXT_ALIGNMENT_LEADING, true)?,
            title: make_fmt(factory, DWRITE_FONT_WEIGHT_SEMI_BOLD, 13.0, DWRITE_TEXT_ALIGNMENT_LEADING, true)?,
            artist: make_fmt(factory, DWRITE_FONT_WEIGHT_NORMAL, 12.0, DWRITE_TEXT_ALIGNMENT_LEADING, true)?,
            body: make_fmt(factory, DWRITE_FONT_WEIGHT_NORMAL, 13.0, DWRITE_TEXT_ALIGNMENT_LEADING, false)?,
            caption: make_fmt(factory, DWRITE_FONT_WEIGHT_NORMAL, 11.5, DWRITE_TEXT_ALIGNMENT_LEADING, true)?,
            caption_center: make_fmt(factory, DWRITE_FONT_WEIGHT_MEDIUM, 11.5, DWRITE_TEXT_ALIGNMENT_CENTER, true)?,
            badge: make_fmt(factory, DWRITE_FONT_WEIGHT_BOLD, 11.0, DWRITE_TEXT_ALIGNMENT_CENTER, true)?,
            clock: make_fmt(factory, DWRITE_FONT_WEIGHT_MEDIUM, 13.0, DWRITE_TEXT_ALIGNMENT_TRAILING, true)?,
        })
    }
}

pub fn draw_text(
    rt: &ID2D1RenderTarget,
    text: &str,
    format: &IDWriteTextFormat,
    brush: &ID2D1SolidColorBrush,
    rect: &D2D_RECT_F,
) {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    // SAFETY: D2D DrawText with verified UTF-16 slice.
    unsafe {
        rt.DrawText(
            &utf16,
            format,
            rect,
            brush,
            D2D1_DRAW_TEXT_OPTIONS_CLIP,
            DWRITE_MEASURING_MODE_NATURAL,
        );
    }
}

pub fn get_date_and_time_parts() -> (String, String) {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    // SAFETY: GetLocalTime populates a stack SYSTEMTIME value.
    let st = unsafe { GetLocalTime() };
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let day_name = days.get(st.wDayOfWeek as usize).unwrap_or(&"Mon");
    let month_name = months.get(st.wMonth.saturating_sub(1) as usize).unwrap_or(&"Nov");
    let (hour_12, am_pm) = match st.wHour {
        0 => (12, "AM"),
        1..=11 => (st.wHour, "AM"),
        12 => (12, "PM"),
        _ => (st.wHour - 12, "PM"),
    };
    (
        format!("{}, {} {}", day_name, month_name, st.wDay),
        format!("{}:{:02} {}", hour_12, st.wMinute, am_pm),
    )
}

pub fn get_current_date_time_string() -> String {
    let (d, t) = get_date_and_time_parts();
    format!("{d}  {t}")
}
