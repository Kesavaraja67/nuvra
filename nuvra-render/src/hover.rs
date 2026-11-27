// nuvra-render/src/hover.rs
// Hover state machine with intent delay & hysteresis — Document UI-1 Section 9.1.
// R-13: No unsafe in this crate.
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverState {
    Collapsed,
    HoverPending,
    Expanding,
    Expanded,
    CollapsePending,
    Collapsing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverEvent {
    PointerEnter,
    PointerLeave,
    IntentTimerFired,
    CollapseTimerFired,
    AnimationSettled,
    ClickToExpand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverAction {
    None,
    StartIntentTimer,
    CancelIntentTimer,
    StartEdgeWake,
    ReverseEdgeWake,
    CommitExpand,
    StartCollapseTimer,
    CancelCollapseTimer,
    CommitCollapse,
    ReverseMidFlight,
}

pub struct HoverStateMachine {
    state: HoverState,
    pointer_inside: bool,
    has_button_down: bool,
    is_shelf_drag_active: bool,
}

impl HoverStateMachine {
    pub fn new() -> Self {
        Self {
            state: HoverState::Collapsed,
            pointer_inside: false,
            has_button_down: false,
            is_shelf_drag_active: false,
        }
    }

    pub fn state(&self) -> HoverState {
        self.state
    }

    pub fn set_button_down(&mut self, is_down: bool) -> HoverAction {
        self.has_button_down = is_down;
        self.re_evaluate_collapse()
    }

    pub fn set_shelf_drag(&mut self, is_active: bool) -> HoverAction {
        self.is_shelf_drag_active = is_active;
        self.re_evaluate_collapse()
    }

    fn re_evaluate_collapse(&mut self) -> HoverAction {
        if self.state == HoverState::Expanded
            && !self.pointer_inside
            && !self.has_button_down
            && !self.is_shelf_drag_active
        {
            self.state = HoverState::CollapsePending;
            HoverAction::StartCollapseTimer
        } else {
            HoverAction::None
        }
    }

    pub fn handle_event(&mut self, event: HoverEvent) -> HoverAction {
        match event {
            HoverEvent::PointerEnter => self.pointer_inside = true,
            HoverEvent::PointerLeave => self.pointer_inside = false,
            _ => {}
        }

        match (self.state, event) {
            (HoverState::Collapsed, HoverEvent::PointerEnter) => {
                self.state = HoverState::HoverPending;
                HoverAction::StartIntentTimer
            }
            (HoverState::HoverPending, HoverEvent::PointerLeave) => {
                self.state = HoverState::Collapsed;
                HoverAction::ReverseEdgeWake
            }
            (HoverState::HoverPending, HoverEvent::IntentTimerFired) => {
                self.state = HoverState::Expanding;
                HoverAction::CommitExpand
            }
            (HoverState::Expanding, HoverEvent::AnimationSettled) => {
                if !self.pointer_inside && !self.has_button_down && !self.is_shelf_drag_active {
                    self.state = HoverState::CollapsePending;
                    HoverAction::StartCollapseTimer
                } else {
                    self.state = HoverState::Expanded;
                    HoverAction::None
                }
            }
            (HoverState::Expanded, HoverEvent::PointerLeave) => {
                if self.has_button_down || self.is_shelf_drag_active {
                    HoverAction::None
                } else {
                    self.state = HoverState::CollapsePending;
                    HoverAction::StartCollapseTimer
                }
            }
            (HoverState::CollapsePending, HoverEvent::PointerEnter) => {
                self.state = HoverState::Expanded;
                HoverAction::CancelCollapseTimer
            }
            (HoverState::CollapsePending, HoverEvent::CollapseTimerFired) => {
                if self.has_button_down || self.is_shelf_drag_active {
                    self.state = HoverState::Expanded;
                    HoverAction::None
                } else {
                    self.state = HoverState::Collapsing;
                    HoverAction::CommitCollapse
                }
            }
            (HoverState::Collapsing, HoverEvent::PointerEnter) => {
                self.state = HoverState::Expanding;
                HoverAction::ReverseMidFlight
            }
            (HoverState::Collapsing, HoverEvent::AnimationSettled) => {
                self.state = HoverState::Collapsed;
                HoverAction::None
            }
            (_, HoverEvent::ClickToExpand) => {
                self.state = HoverState::Expanding;
                HoverAction::CommitExpand
            }
            _ => HoverAction::None,
        }
    }
}

impl Default for HoverStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_state_transitions() {
        let mut sm = HoverStateMachine::new();
        assert_eq!(sm.state(), HoverState::Collapsed);

        let action = sm.handle_event(HoverEvent::PointerEnter);
        assert_eq!(sm.state(), HoverState::HoverPending);
        assert_eq!(action, HoverAction::StartIntentTimer);

        let action = sm.handle_event(HoverEvent::IntentTimerFired);
        assert_eq!(sm.state(), HoverState::Expanding);
        assert_eq!(action, HoverAction::CommitExpand);

        let action = sm.handle_event(HoverEvent::AnimationSettled);
        assert_eq!(sm.state(), HoverState::Expanded);
        assert_eq!(action, HoverAction::None);

        let action = sm.handle_event(HoverEvent::PointerLeave);
        assert_eq!(sm.state(), HoverState::CollapsePending);
        assert_eq!(action, HoverAction::StartCollapseTimer);

        let action = sm.handle_event(HoverEvent::CollapseTimerFired);
        assert_eq!(sm.state(), HoverState::Collapsing);
        assert_eq!(action, HoverAction::CommitCollapse);

        let action = sm.handle_event(HoverEvent::AnimationSettled);
        assert_eq!(sm.state(), HoverState::Collapsed);
        assert_eq!(action, HoverAction::None);
    }

    #[test]
    fn test_pointer_leave_during_expansion_and_inhibitor_release_collapse() {
        let mut sm = HoverStateMachine::new();
        assert_eq!(
            sm.handle_event(HoverEvent::PointerEnter),
            HoverAction::StartIntentTimer
        );
        assert_eq!(
            sm.handle_event(HoverEvent::IntentTimerFired),
            HoverAction::CommitExpand
        );
        assert_eq!(sm.handle_event(HoverEvent::PointerLeave), HoverAction::None);
        assert_eq!(
            sm.handle_event(HoverEvent::AnimationSettled),
            HoverAction::StartCollapseTimer
        );
        assert_eq!(sm.state(), HoverState::CollapsePending);

        // Re-enter while pending, hold button, leave outside, release button -> starts collapse
        assert_eq!(
            sm.handle_event(HoverEvent::PointerEnter),
            HoverAction::CancelCollapseTimer
        );
        assert_eq!(sm.set_button_down(true), HoverAction::None);
        assert_eq!(sm.handle_event(HoverEvent::PointerLeave), HoverAction::None);
        assert_eq!(sm.state(), HoverState::Expanded);
        assert_eq!(sm.set_button_down(false), HoverAction::StartCollapseTimer);
        assert_eq!(sm.state(), HoverState::CollapsePending);

        // Suppressed CollapseTimerFired during shelf drag resumes collapse when drag ends
        assert_eq!(sm.set_shelf_drag(true), HoverAction::None);
        assert_eq!(
            sm.handle_event(HoverEvent::CollapseTimerFired),
            HoverAction::None
        );
        assert_eq!(sm.state(), HoverState::Expanded);
        assert_eq!(sm.set_shelf_drag(false), HoverAction::StartCollapseTimer);
        assert_eq!(sm.state(), HoverState::CollapsePending);
    }
}
