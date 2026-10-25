//! TextField: a styled wrapper over `gpui_base`'s input.
//!
//! ## Why there is almost no code here
//!
//! `gpui_base::input` already implements the entire text-editing engine: rope
//! storage, cursor, selection, movement, touch selection, masking, undo history,
//! native IME integration and painting (see
//! `gpui-base/src/input/base/state.rs`, ~483 KB). Its `Input` element is
//! documented as *unstyled* on purpose: "Applications that need a fully styled
//! control can wrap this state with their own presentation".
//!
//! That is exactly this module's job. Everything visual comes from our tokens;
//! everything behavioural comes from upstream. Writing the engine ourselves was
//! the mistake recorded in AGENTS.md Q33.

use gpui::{
    App, Context, Entity, Focusable, IntoElement, RenderOnce, SharedString, Window, div,
    prelude::*, px, rgb,
};

use crate::controls::{
    ControlState, StateInput, derive_state, stroke_color, surface_fill, text_color,
};
use crate::tokens::{Palette, radius, space, typography};

/// Re-exported so screens create the state through our facade, not by importing
/// `gpui_base` themselves.
pub use gpui_kit::base::input::{Input, InputState};

/// Build a single-line input state with upstream defaults.
///
/// `window` is required by the upstream constructor, which wires the text system
/// and IME hooks.
pub fn text_input(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
    InputState::new(window, cx)
}

/// Paint a styled text field around an upstream [`InputState`].
#[derive(IntoElement)]
pub struct TextField {
    id: &'static str,
    state: Entity<InputState>,
    palette: Palette,
    visual: Option<ControlState>,
    helper: Option<SharedString>,
    invalid: bool,
    width: f32,
}

impl TextField {
    pub const DEFAULT_WIDTH: f32 = 240.0;

    pub fn new(id: &'static str, state: &Entity<InputState>, palette: Palette) -> Self {
        Self {
            id,
            state: state.clone(),
            palette,
            visual: None,
            helper: None,
            invalid: false,
            width: Self::DEFAULT_WIDTH,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn helper(mut self, text: impl Into<SharedString>) -> Self {
        self.helper = Some(text.into());
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Force a visual state instead of deriving it from focus.
    pub fn visual(mut self, visual: ControlState) -> Self {
        self.visual = Some(visual);
        self
    }

    pub fn from_inputs(mut self, input: StateInput) -> Self {
        self.visual = Some(derive_state(input));
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let token = typography::control::BODY;
        let height = token.line_box(space::CONTROL_Y);

        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);

        // Derive the visual state only when the caller did not force one.
        let visual = self.visual.unwrap_or(if self.invalid {
            ControlState::Invalid
        } else if focused {
            ControlState::Focused
        } else {
            ControlState::Resting
        });

        let stroke = stroke_color(visual, &self.palette);

        // Fixed-width border, colour-only state change: swapping width on focus
        // would reflow the row and make the text jitter.
        let field = div()
            .id(self.id)
            .w(px(self.width))
            .h(px(height))
            .px(px(space::CONTROL_X))
            .flex()
            .items_center()
            .rounded(px(radius::CONTROL))
            .border_1()
            .border_color(match stroke {
                Some(c) => rgb(c),
                None => rgb(self.palette.surface.card),
            })
            .bg(rgb(surface_fill(visual, &self.palette)))
            // The upstream input is unstyled: it inherits this text style and
            // fills the remaining width.
            .text_size(px(token.line_height()))
            .text_color(rgb(text_color(visual, &self.palette)))
            .child(Input::new(&self.state));

        let color = if self.invalid {
            self.palette.border_danger
        } else {
            self.palette.text_secondary
        };
        let helper_token = typography::control::HELPER;

        match self.helper {
            None => field.into_any_element(),
            Some(helper) => div()
                .flex()
                .flex_col()
                .gap(px(space::INLINE))
                .w(px(self.width))
                .child(field)
                .child(
                    div()
                        .text_size(px(helper_token.line_height()))
                        .text_color(rgb(color))
                        .child(helper),
                )
                .into_any_element(),
        }
    }
}
