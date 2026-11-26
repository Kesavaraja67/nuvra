// nuvra-render/tests/layout_test.rs
// Integration and unit tests for pure layout computation — Document UI-1 Section 6 & 14.

use nuvra_render::layout::{compute_layout, ItemId, LayoutState};

#[test]
fn test_layout_dimensions_and_even_width() {
    for scale in [1.0, 1.25, 1.5, 2.0, 3.0] {
        let l_collapsed = compute_layout(LayoutState::Collapsed, scale, 149.0, false, 0, None);
        assert_eq!(
            (l_collapsed.body.width() as i32) % 2,
            0,
            "Collapsed body width must be even at scale {scale}"
        );

        let l_expanded_media = compute_layout(LayoutState::Expanded, scale, 149.0, true, 2, None);
        assert_eq!(
            (l_expanded_media.body.width() as i32) % 2,
            0,
            "Expanded media width must be even at scale {scale}"
        );

        let l_expanded_nomedia =
            compute_layout(LayoutState::Expanded, scale, 149.0, false, 0, None);
        assert_eq!(
            (l_expanded_nomedia.body.width() as i32) % 2,
            0,
            "Expanded no-media width must be even at scale {scale}"
        );
    }
}

#[test]
fn test_no_overlapping_items_in_toolbar() {
    let layout = compute_layout(LayoutState::Expanded, 1.0, 149.0, true, 2, None);
    let items = &layout.items;

    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            let a = &items[i];
            let b = &items[j];
            if a.id == ItemId::Badge
                || b.id == ItemId::Badge
                || a.id == ItemId::DrawerContainer
                || b.id == ItemId::DrawerContainer
            {
                continue; // Badge overlaps Bell by design
            }
            // Check non-overlap in visual rects
            let overlaps = a.visual_rect.left < b.visual_rect.right
                && a.visual_rect.right > b.visual_rect.left
                && a.visual_rect.top < b.visual_rect.bottom
                && a.visual_rect.bottom > b.visual_rect.top;
            assert!(
                !overlaps,
                "Visual overlap detected between {:?} and {:?}",
                a.id, b.id
            );
        }
    }
}

#[test]
fn test_corner_clearance() {
    let layout = compute_layout(LayoutState::Expanded, 1.0, 149.0, true, 2, None);
    for item in &layout.items {
        if item.id == ItemId::DrawerContainer {
            continue;
        }
        assert!(
            item.visual_rect.left >= 12.0,
            "{:?} left edge too close to corner",
            item.id
        );
        assert!(
            item.visual_rect.right <= layout.body.width() - 12.0,
            "{:?} right edge too close to corner",
            item.id
        );
    }
}
