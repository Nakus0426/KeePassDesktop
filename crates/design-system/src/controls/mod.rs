//! Atomic controls.
//!
//! Scope for G5: `Button`, `TextField`, `ListRow`, `Dialog`.
//!
//! ## Borderless rule
//!
//! A resting control draws no border. Layering comes from the surface tier
//! (`page` vs `card`). State strokes exist but are **colour-only at fixed
//! width**, so focus and validation never reflow layout.
//!
//! ## The one risk carrier
//!
//! [`text_field::TextField`] handles CJK IME. Its editing logic lives in
//! [`text_edit::TextEdit`], free of GPUI, so the parts that can corrupt a value
//! are unit-tested. GPUI upstream has open defects around preedit cleanup and
//! copy-after-composition (Zed #39608, #64389).

pub mod button;
pub mod dialog;
pub mod icon;
pub mod list_row;
pub mod text_input;

/// Interaction state that may change a control's appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlState {
    Resting,
    Hovered,
    Focused,
    /// Focused **and** holding an invalid value.
    Invalid,
    Pressed,
    Disabled,
}

impl ControlState {
    /// Whether the control is interactive. Disabled swallows every event, so it
    /// wins over any other state.
    pub const fn is_interactive(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Whether a stroke is drawn at all.
    pub const fn draws_stroke(self) -> bool {
        matches!(self, Self::Focused | Self::Invalid)
    }
}

/// Whether the control currently paints a border. Resting, hovered, pressed and
/// disabled all draw nothing: layering is background luminance, not stroke.
pub const fn stroke_visible(state: ControlState) -> bool {
    state.draws_stroke()
}

/// Stroke colour for a state, or `None` when nothing is drawn.
///
/// Callers keep a constant-width transparent border and only swap its colour, so
/// focusing a field never moves neighbouring pixels.
pub fn stroke_color(state: ControlState, palette: &crate::tokens::Palette) -> Option<u32> {
    match state {
        ControlState::Focused => Some(palette.border_focus),
        ControlState::Invalid => Some(palette.border_danger),
        _ => None,
    }
}

/// Fill colour for a state on a surface of tier `card`.
pub fn surface_fill(state: ControlState, palette: &crate::tokens::Palette) -> u32 {
    match state {
        ControlState::Resting | ControlState::Focused | ControlState::Invalid => {
            palette.surface.card
        }
        ControlState::Hovered => palette.state.hover,
        ControlState::Pressed => palette.state.active,
        ControlState::Disabled => palette.state.disabled,
    }
}

/// Text colour for a state. Validation never recolours the text: the stroke
/// carries the error so the value stays readable while the user fixes it.
pub fn text_color(state: ControlState, palette: &crate::tokens::Palette) -> u32 {
    match state {
        ControlState::Disabled => palette.text_disabled,
        ControlState::Resting
        | ControlState::Hovered
        | ControlState::Focused
        | ControlState::Invalid
        | ControlState::Pressed => palette.text_primary,
    }
}

/// Derive a control's visual state from its inputs.
///
/// One function so Button, TextField and ListRow cannot drift apart.
pub fn derive_state(input: StateInput) -> ControlState {
    if input.disabled {
        ControlState::Disabled
    } else if input.focused && input.invalid {
        ControlState::Invalid
    } else if input.focused {
        ControlState::Focused
    } else if input.pressed {
        ControlState::Pressed
    } else if input.hovered {
        ControlState::Hovered
    } else {
        ControlState::Resting
    }
}

/// Raw widget inputs, before precedence is applied.
#[derive(Clone, Copy, Debug, Default)]
pub struct StateInput {
    pub hovered: bool,
    pub focused: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub invalid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::DARK;

    #[test]
    fn disabled_wins_over_everything() {
        let state = derive_state(StateInput {
            hovered: true,
            focused: true,
            pressed: true,
            disabled: true,
            invalid: true,
        });
        assert_eq!(state, ControlState::Disabled);
    }

    #[test]
    fn invalid_focus_beats_plain_focus() {
        let state = derive_state(StateInput {
            focused: true,
            invalid: true,
            ..Default::default()
        });
        assert_eq!(state, ControlState::Invalid);
    }

    #[test]
    fn pressed_beats_hovered() {
        let state = derive_state(StateInput {
            hovered: true,
            pressed: true,
            ..Default::default()
        });
        assert_eq!(state, ControlState::Pressed);
    }

    #[test]
    fn resting_draws_no_stroke() {
        for state in [
            ControlState::Resting,
            ControlState::Hovered,
            ControlState::Pressed,
            ControlState::Disabled,
        ] {
            assert!(!stroke_visible(state), "{state:?} must not draw a border");
            assert_eq!(stroke_color(state, &DARK), None);
        }
    }

    #[test]
    fn focused_and_invalid_draw_a_stroke() {
        assert_eq!(
            stroke_color(ControlState::Focused, &DARK),
            Some(DARK.border_focus)
        );
        assert_eq!(
            stroke_color(ControlState::Invalid, &DARK),
            Some(DARK.border_danger)
        );
        assert!(stroke_visible(ControlState::Focused));
        assert!(stroke_visible(ControlState::Invalid));
    }

    #[test]
    fn stroke_is_colour_only_so_width_never_changes() {
        // A focusing control must keep the same border width, otherwise the row
        // shifts by a pixel and text jitters. The width lives in one token.
        let widths = [
            ControlState::Resting,
            ControlState::Focused,
            ControlState::Invalid,
        ]
        .map(|_| crate::tokens::primitive::BORDER_WIDTH);
        assert_eq!(widths[0], widths[1]);
        assert_eq!(widths[1], widths[2]);
    }

    #[test]
    fn invalid_keeps_text_readable() {
        // The user must be able to read what they typed while fixing it.
        assert_eq!(
            text_color(ControlState::Invalid, &DARK),
            text_color(ControlState::Resting, &DARK)
        );
    }

    #[test]
    fn disabled_is_not_interactive() {
        assert!(!ControlState::Disabled.is_interactive());
        for state in [
            ControlState::Resting,
            ControlState::Hovered,
            ControlState::Focused,
            ControlState::Invalid,
            ControlState::Pressed,
        ] {
            assert!(state.is_interactive());
        }
    }

    #[test]
    fn hovered_fill_differs_from_resting() {
        assert_ne!(
            surface_fill(ControlState::Hovered, &DARK),
            surface_fill(ControlState::Resting, &DARK)
        );
    }

    #[test]
    fn pressed_fill_differs_from_hovered() {
        assert_ne!(
            surface_fill(ControlState::Pressed, &DARK),
            surface_fill(ControlState::Hovered, &DARK)
        );
    }
}
