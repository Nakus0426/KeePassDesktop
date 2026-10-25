//! Button: a styled wrapper over `gpui_base`'s button.
//!
//! ## What upstream provides
//!
//! `gpui_base::Button` already implements the behaviour that is easy to get
//! wrong: focus handling and tab order (`tab_index`, `tab_stop`, `focusable`),
//! keyboard activation, pointer press/release semantics, the disabled path, and
//! the accessibility role. It implements `Styled`, so appearance is ours.
//!
//! Per AGENTS.md Q33 we therefore package it instead of drawing our own box: our
//! only job is mapping `ButtonKind`/`ButtonSize`/`ControlState` onto colours and
//! metrics from the token layer.

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, prelude::*, px, rgb,
};

use crate::controls::{ControlState, StateInput, derive_state};
use crate::tokens::{DARK, Palette, radius, space, typography};

/// Re-exported so screens get the upstream element through our facade.
pub use gpui_kit::base::Button as UpstreamButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    /// Filled with the accent colour. At most one per view region.
    Primary,
    /// Card-coloured, for secondary actions.
    Secondary,
    /// No fill at rest; used inside dense rows.
    Ghost,
    /// Destructive action.
    Danger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    const fn token(self) -> crate::tokens::TypeToken {
        match self {
            Self::Small => typography::control::LABEL,
            Self::Medium => typography::control::BUTTON,
            Self::Large => typography::BODY_L,
        }
    }

    /// Horizontal padding scales with the size, staying on the 8px grid.
    const fn padding_x(self) -> f32 {
        match self {
            Self::Small => space::INLINE,
            Self::Medium => space::CONTROL_X,
            Self::Large => space::STACK,
        }
    }
}

/// Click handler signature, aliased so the field type stays readable.
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Paint a token-styled button around an upstream `gpui_base::Button`.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    kind: ButtonKind,
    size: ButtonSize,
    state: ControlState,
    palette: Palette,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: ButtonKind::Secondary,
            size: ButtonSize::Medium,
            state: ControlState::Resting,
            palette: DARK,
            on_click: None,
        }
    }

    pub fn primary(mut self) -> Self {
        self.kind = ButtonKind::Primary;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.kind = ButtonKind::Ghost;
        self
    }

    pub fn danger(mut self) -> Self {
        self.kind = ButtonKind::Danger;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = ButtonSize::Large;
        self
    }

    /// Set the visual state explicitly. Precedence lives in [`derive_state`].
    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }

    pub fn from_inputs(mut self, input: StateInput) -> Self {
        self.state = derive_state(input);
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let token = self.size.token();
        let height = token.line_box(space::CONTROL_Y);
        let padding_x = self.size.padding_x();
        let disabled = self.state == ControlState::Disabled;

        let hovered = self.state == ControlState::Hovered;
        let pressed = self.state == ControlState::Pressed;

        let (fill, fg) = match self.kind {
            ButtonKind::Primary => (
                if disabled {
                    self.palette.state.disabled
                } else if hovered || pressed {
                    self.palette.accent_hover
                } else {
                    self.palette.accent
                },
                if disabled {
                    self.palette.text_disabled
                } else {
                    self.palette.text_on_accent
                },
            ),
            ButtonKind::Danger => (
                if disabled {
                    self.palette.state.disabled
                } else if hovered || pressed {
                    self.palette.state.active
                } else {
                    self.palette.danger
                },
                if disabled {
                    self.palette.text_disabled
                } else {
                    self.palette.text_on_accent
                },
            ),
            ButtonKind::Secondary => (
                if disabled {
                    self.palette.state.disabled
                } else if pressed {
                    self.palette.state.active
                } else if hovered {
                    self.palette.state.hover
                } else {
                    self.palette.surface.card
                },
                if disabled {
                    self.palette.text_disabled
                } else {
                    self.palette.text_primary
                },
            ),
            ButtonKind::Ghost => (
                if disabled {
                    self.palette.surface.page
                } else if pressed {
                    self.palette.state.active
                } else if hovered {
                    self.palette.state.hover
                } else {
                    self.palette.surface.page
                },
                if disabled {
                    self.palette.text_disabled
                } else {
                    self.palette.text_primary
                },
            ),
        };

        // Upstream owns focus, tab order, keyboard activation and the disabled
        // path; we only override the visual layer.
        //
        // The label is a bare text child (not `div().child(label)`): a wrapping
        // div starts its own style context and does not inherit our
        // `text_color`, which silently greys out every label.
        let mut button = UpstreamButton::new(self.id)
            .disabled(disabled)
            .h(px(height))
            .px(px(padding_x))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius::SURFACE))
            .bg(rgb(fill))
            .text_size(px(token.line_height()))
            .text_color(rgb(fg))
            .child(self.label);

        if !disabled && let Some(handler) = self.on_click {
            button = button.on_click(handler);
        }

        button
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::primitive;

    #[test]
    fn button_height_follows_the_type_token() {
        // A button shorter than its line box clips descenders at some DPI scale.
        for size in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
            let token = size.token();
            let height = token.line_box(space::CONTROL_Y);
            assert!(height >= token.line_height());
            assert_eq!(height % 2.0, 0.0, "height should stay on the pixel grid");
        }
    }

    #[test]
    fn button_radii_come_from_tokens() {
        assert_eq!(radius::SURFACE, primitive::RADIUS_MD);
    }

    #[test]
    fn padding_scales_with_size() {
        assert!(ButtonSize::Small.padding_x() < ButtonSize::Medium.padding_x());
        assert!(ButtonSize::Medium.padding_x() < ButtonSize::Large.padding_x());
    }

    #[test]
    fn padding_stays_on_the_four_px_grid() {
        for size in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large] {
            assert_eq!(size.padding_x() % 4.0, 0.0);
        }
    }

    #[test]
    fn disabled_is_not_interactive() {
        let input = StateInput {
            disabled: true,
            ..Default::default()
        };
        assert!(!derive_state(input).is_interactive());
    }

    #[test]
    fn pressed_beats_hovered_for_visual_state() {
        let input = StateInput {
            hovered: true,
            pressed: true,
            ..Default::default()
        };
        assert_eq!(derive_state(input), ControlState::Pressed);
    }
}
