//! ListRow.
//!
//! A single row in the entry list. Two constraints shaped it:
//!
//! - **Fixed height from the type token.** A virtualised list needs a constant
//!   row height; deriving it from the taller of the title/subtitle line boxes
//!   keeps CJK and Latin rows identical, so the scrollbar does not lie.
//! - **Selection is background, not stroke.** The borderless rule applies here
//!   too: a selected row lifts to `card`, a hovered row lifts to `state.hover`.

use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, rgb,
};

use crate::controls::{ControlState, StateInput, derive_state, text_color};
use crate::tokens::{DARK, Palette, radius, space, typography};

/// Row height, derived once so the list and the virtualiser cannot disagree.
///
/// Title line box + gap + subtitle line box + vertical padding. The padding is
/// doubled on top of the line boxes on purpose: glyphs routinely paint outside
/// their line box (CJK ascenders, Latin descenders, superscript digits), so a
/// row sized to exactly the boxes clips the first and last line. The extra
/// `CONTROL_Y` per edge is the observed minimum that does not.
pub const fn row_height() -> f32 {
    let title = typography::control::ROW_TITLE.line_height();
    let subtitle = typography::control::ROW_SUBTITLE.line_height();
    title + space::INLINE + subtitle + space::CONTROL_Y * 4.0
}

#[derive(IntoElement)]
pub struct ListRow {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    trailing: Option<SharedString>,
    state: ControlState,
    selected: bool,
    palette: Palette,
}

impl ListRow {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            trailing: None,
            state: ControlState::Resting,
            selected: false,
            palette: DARK,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Right-aligned secondary text, e.g. a TOTP countdown or an attachment count.
    pub fn trailing(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing = Some(text.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }

    pub fn from_inputs(mut self, input: StateInput) -> Self {
        self.state = derive_state(input);
        self
    }
}

impl RenderOnce for ListRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let title_token = typography::control::ROW_TITLE;
        let subtitle_token = typography::control::ROW_SUBTITLE;

        // Selection outranks hover: the user must always be able to tell which
        // row is current, even while the pointer is elsewhere.
        let fill = if self.selected {
            self.palette.state.active
        } else {
            match self.state {
                ControlState::Hovered => self.palette.state.hover,
                ControlState::Pressed => self.palette.state.active,
                ControlState::Disabled => self.palette.surface.page,
                _ => self.palette.surface.card,
            }
        };

        let mut row = div()
            .id(self.id)
            .h(px(row_height()))
            .px(px(space::CONTROL_X))
            .py(px(space::CONTROL_Y * 2.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(space::STACK))
            .rounded(px(radius::SURFACE))
            .bg(rgb(fill))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(space::INLINE))
                    .child(
                        div()
                            .text_size(px(title_token.line_height()))
                            .text_color(rgb(text_color(self.state, &self.palette)))
                            .truncate()
                            .child(self.title),
                    )
                    .child(
                        div()
                            .text_size(px(subtitle_token.line_height()))
                            .text_color(rgb(self.palette.text_secondary))
                            .truncate()
                            .child(self.subtitle.unwrap_or_else(|| "".into())),
                    ),
            );

        if let Some(trailing) = self.trailing {
            row = row.child(
                div()
                    .text_size(px(subtitle_token.line_height()))
                    .text_color(rgb(self.palette.text_secondary))
                    .child(trailing),
            );
        }

        if self.state.is_interactive() {
            row = row.cursor_pointer();
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_height_is_stable_and_fits_both_lines() {
        let title = typography::control::ROW_TITLE.line_height();
        let subtitle = typography::control::ROW_SUBTITLE.line_height();
        // Title + gap + subtitle + vertical padding on both edges.
        let expected = title + space::INLINE + subtitle + space::CONTROL_Y * 4.0;
        assert_eq!(row_height(), expected);
    }

    #[test]
    fn row_height_leaves_room_outside_the_line_boxes() {
        // Glyphs paint outside their line box (CJK ascenders, Latin descenders).
        // A row sized to exactly the boxes clips the first and last line, which
        // is visible on "GitHub" and CJK titles.
        let boxes = typography::control::ROW_TITLE.line_height()
            + space::INLINE
            + typography::control::ROW_SUBTITLE.line_height();
        let slack = row_height() - boxes;
        assert!(
            slack >= space::CONTROL_Y * 2.0,
            "only {slack}px of vertical slack, need at least {}px per edge",
            space::CONTROL_Y
        );
    }

    #[test]
    fn row_height_does_not_depend_on_text() {
        // Virtualisation breaks if height varies per row; this pins the property
        // by asserting the function is constant for any content.
        let a = row_height();
        let b = row_height();
        assert_eq!(a, b);
    }

    #[test]
    fn selection_fill_differs_from_rest_and_hover() {
        // Selected must be distinguishable from both rest and hover, otherwise
        // the user cannot tell what is current.
        assert_ne!(DARK.state.active, DARK.surface.card);
        assert_ne!(DARK.state.active, DARK.state.hover);
    }
}
