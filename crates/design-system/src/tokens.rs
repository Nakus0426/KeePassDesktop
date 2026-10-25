//! Design tokens.
//!
//! Three layers, one direction: `primitive` (raw numbers) → [`semantic`] (roles)
//! → controls. A control must never contain a raw colour or size literal.
//!
//! ## Why type tokens carry two metric sets
//!
//! A CJK glyph fills its em box; a Latin glyph's visual height is the x-height or
//! cap height, much shorter. At the same `size`, a shared line-height ratio gives
//! CJK lines too little air and Latin lines too much. So every type token stores
//! `cjk` and `latin` metrics independently, in absolute px, and containers size
//! themselves from [`TypeToken::line_height`] (the taller of the two).

/// Raw scale. Controls must not reference these directly; go through [`semantic`].
pub mod primitive {
    /// Base unit is 4px; every spacing value is a multiple of it.
    pub const SPACE_1: f32 = 4.0;
    pub const SPACE_2: f32 = 8.0;
    pub const SPACE_3: f32 = 12.0;
    pub const SPACE_4: f32 = 16.0;
    pub const SPACE_6: f32 = 24.0;
    pub const SPACE_8: f32 = 32.0;
    pub const SPACE_12: f32 = 48.0;

    pub const RADIUS_SM: f32 = 6.0;
    pub const RADIUS_MD: f32 = 10.0;
    pub const RADIUS_LG: f32 = 14.0;

    pub const DURATION_FAST_MS: u64 = 120;
    pub const DURATION_BASE_MS: u64 = 180;

    /// Fixed width, so a state change never reflows layout.
    pub const BORDER_WIDTH: f32 = 1.0;
}

/// Semantic spacing roles. `gpui-kit`'s theme persists colours and radii but not
/// a custom spacing scale, so this scale is ours to own.
pub mod space {
    pub const INLINE: f32 = super::primitive::SPACE_2;
    pub const CONTROL_X: f32 = super::primitive::SPACE_3;
    pub const CONTROL_Y: f32 = super::primitive::SPACE_2;
    pub const STACK: f32 = super::primitive::SPACE_4;
    pub const SECTION: f32 = super::primitive::SPACE_6;
    pub const PAGE: f32 = super::primitive::SPACE_8;
}

/// Icon sizes. Separate from [`typography`] on purpose: an icon is measured by its
/// square box, not by a line height, and it must stay on the 4px grid.
pub mod icon {
    /// Inline with text, inside a small button.
    pub const SM: f32 = super::primitive::SPACE_4;
    /// Default: list rows, toolbars. Slightly larger than `BODY_M`'s font size so
    /// the visual weight of a filled shape matches adjacent 14px text.
    pub const MD: f32 = 20.0;
    /// Empty states and dialog headers.
    pub const LG: f32 = super::primitive::SPACE_6;
}

/// Semantic radii.
pub mod radius {
    /// Text fields, small controls.
    pub const CONTROL: f32 = super::primitive::RADIUS_SM;
    /// Buttons, list rows.
    pub const SURFACE: f32 = super::primitive::RADIUS_MD;
    /// Cards and dialogs.
    pub const CARD: f32 = super::primitive::RADIUS_LG;
}

/// One text style, resolved for a single script.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub size: f32,
    /// Absolute px, not a multiplier (see the module doc).
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl TextMetrics {
    /// Height of a single-line box holding this text plus vertical padding.
    pub const fn box_height(&self, padding_y: f32) -> f32 {
        self.line_height + padding_y * 2.0
    }
}

/// A type token: the same style for both scripts, with independent metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeToken {
    pub cjk: TextMetrics,
    pub latin: TextMetrics,
}

impl TypeToken {
    /// Taller of the two line heights. Use for any container that may hold mixed
    /// script text, so nothing clips.
    pub const fn line_height(&self) -> f32 {
        if self.cjk.line_height > self.latin.line_height {
            self.cjk.line_height
        } else {
            self.latin.line_height
        }
    }

    /// Height of a single-line control holding this text plus `padding_y`.
    pub const fn line_box(&self, padding_y: f32) -> f32 {
        self.line_height() + padding_y * 2.0
    }

    /// Metrics for a run of text. Pure ASCII gets the tighter Latin metrics;
    /// anything else is treated as CJK, because a mixed run must use the taller
    /// box or the CJK glyphs clip.
    pub const fn metrics_for(&self, text: &str) -> &TextMetrics {
        if is_ascii(text) {
            &self.latin
        } else {
            &self.cjk
        }
    }
}

/// Const-fn ASCII check, usable in `const` context and without allocation.
pub const fn is_ascii(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] >= 0x80 {
            return false;
        }
        i += 1;
    }
    true
}

const fn token(cjk_lh: f32, latin_lh: f32, cjk_track: f32, size: f32) -> TypeToken {
    TypeToken {
        cjk: TextMetrics {
            size,
            line_height: cjk_lh,
            letter_spacing: cjk_track,
        },
        latin: TextMetrics {
            size,
            line_height: latin_lh,
            letter_spacing: 0.0,
        },
    }
}

/// Type scale. CJK line heights run looser than Latin on purpose.
pub mod typography {
    use super::TypeToken;

    pub const CAPTION: TypeToken = super::token(18.0, 16.0, 0.0, 12.0);
    pub const BODY_S: TypeToken = super::token(22.0, 20.0, 0.0, 13.0);
    pub const BODY_M: TypeToken = super::token(24.0, 22.0, 0.0, 14.0);
    pub const BODY_L: TypeToken = super::token(26.0, 24.0, 0.0, 16.0);
    pub const TITLE_S: TypeToken = super::token(30.0, 28.0, 0.0, 18.0);
    pub const TITLE_M: TypeToken = super::token(36.0, 32.0, 0.0, 22.0);

    /// The token a given control uses. One place, so a control never picks.
    pub mod control {
        pub const BODY: super::TypeToken = super::BODY_M;
        pub const LABEL: super::TypeToken = super::BODY_S;
        pub const HELPER: super::TypeToken = super::CAPTION;
        pub const BUTTON: super::TypeToken = super::BODY_M;
        pub const ROW_TITLE: super::TypeToken = super::BODY_M;
        pub const ROW_SUBTITLE: super::TypeToken = super::CAPTION;
        pub const DIALOG_TITLE: super::TypeToken = super::TITLE_S;
    }
}

/// Surface roles. Layering is expressed by background luminance, never by a
/// border, so a container never shares a tier with its page.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceTier {
    /// Window background.
    pub page: u32,
    /// Cards, inputs and list rows resting on the page.
    pub card: u32,
    /// Anything raised above another card: dialogs, sheets, tooltips.
    pub raised: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct StateColors {
    /// Hover on an interactive surface.
    pub hover: u32,
    /// Pressed / active.
    pub active: u32,
    /// Disabled fill.
    pub disabled: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub surface: SurfaceTier,
    pub state: StateColors,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_disabled: u32,
    /// Text on an accent-filled control.
    pub text_on_accent: u32,
    pub border_focus: u32,
    pub border_danger: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub danger: u32,
    /// Dimming layer behind a modal.
    pub scrim: u32,
}

pub const DARK: Palette = Palette {
    surface: SurfaceTier {
        page: 0x111315,
        card: 0x1A1D20,
        raised: 0x23272B,
    },
    state: StateColors {
        hover: 0x2A2F34,
        active: 0x333A40,
        disabled: 0x1A1D20,
    },
    text_primary: 0xECEDEE,
    text_secondary: 0x9BA1A6,
    text_disabled: 0x5C6166,
    text_on_accent: 0xFFFFFF,
    border_focus: 0x4C8DFF,
    border_danger: 0xE5484D,
    accent: 0x4C8DFF,
    accent_hover: 0x6AA1FF,
    danger: 0xE5484D,
    scrim: 0x000000,
};

pub const LIGHT: Palette = Palette {
    surface: SurfaceTier {
        page: 0xF4F5F6,
        card: 0xFFFFFF,
        raised: 0xF0F1F3,
    },
    state: StateColors {
        hover: 0xE9EBED,
        active: 0xDDE0E3,
        disabled: 0xEDEEF0,
    },
    text_primary: 0x1A1D20,
    text_secondary: 0x5C6166,
    text_disabled: 0x9BA1A6,
    text_on_accent: 0xFFFFFF,
    border_focus: 0x2563EB,
    border_danger: 0xD13438,
    accent: 0x2563EB,
    accent_hover: 0x1D4FD7,
    danger: 0xD13438,
    scrim: 0x000000,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_line_height_is_never_tighter_than_latin() {
        for (name, t) in [
            ("CAPTION", typography::CAPTION),
            ("BODY_S", typography::BODY_S),
            ("BODY_M", typography::BODY_M),
            ("BODY_L", typography::BODY_L),
            ("TITLE_S", typography::TITLE_S),
            ("TITLE_M", typography::TITLE_M),
        ] {
            assert!(
                t.cjk.line_height > t.latin.line_height,
                "{name}: CJK must get more leading than Latin"
            );
            assert_eq!(t.line_height(), t.cjk.line_height, "{name}");
        }
    }

    #[test]
    fn sizes_match_between_scripts() {
        for t in [
            typography::CAPTION,
            typography::BODY_S,
            typography::BODY_M,
            typography::BODY_L,
            typography::TITLE_S,
            typography::TITLE_M,
        ] {
            assert_eq!(
                t.cjk.size, t.latin.size,
                "one token must not have two sizes"
            );
        }
    }

    #[test]
    fn line_height_exceeds_font_size() {
        // Below 1.0 the glyphs visibly clip; every token must clear it.
        for t in [
            typography::CAPTION,
            typography::BODY_S,
            typography::BODY_M,
            typography::BODY_L,
            typography::TITLE_S,
            typography::TITLE_M,
        ] {
            assert!(t.latin.line_height > t.latin.size);
            assert!(
                t.cjk.line_height / t.cjk.size >= 1.3,
                "CJK leading below 1.3 looks cramped at 100% scaling"
            );
        }
    }

    #[test]
    fn is_ascii_detects_wide_chars() {
        assert!(is_ascii("Hello"));
        assert!(is_ascii("abc 123"));
        assert!(!is_ascii("你好"));
        assert!(!is_ascii("mixed 中"));
        // Full-width punctuation counts as non-ASCII.
        assert!(!is_ascii("，"));
    }

    #[test]
    fn metrics_for_picks_the_taller_box_for_mixed_text() {
        let t = typography::BODY_M;
        assert_eq!(t.metrics_for("plain latin"), &t.latin);
        assert_eq!(t.metrics_for("中文"), &t.cjk);
        // Mixed text must not use the Latin box or the CJK glyphs clip.
        assert_eq!(t.metrics_for("Hello 世界"), &t.cjk);
    }

    #[test]
    fn line_box_accounts_for_padding_on_both_sides() {
        let t = typography::BODY_M;
        let padding = space::CONTROL_Y;
        assert_eq!(t.line_box(padding), t.line_height() + padding * 2.0);
        assert_eq!(t.line_box(0.0), t.line_height());
    }

    #[test]
    fn text_metrics_box_height_matches_line_box_for_one_script() {
        let t = typography::BODY_M;
        assert_eq!(
            t.cjk.box_height(space::CONTROL_Y),
            t.line_box(space::CONTROL_Y)
        );
    }

    #[test]
    fn every_spacing_token_is_a_multiple_of_four() {
        for (name, v) in [
            ("INLINE", space::INLINE),
            ("CONTROL_X", space::CONTROL_X),
            ("CONTROL_Y", space::CONTROL_Y),
            ("STACK", space::STACK),
            ("SECTION", space::SECTION),
            ("PAGE", space::PAGE),
        ] {
            assert_eq!(v % 4.0, 0.0, "{name} = {v} is off the 4px grid");
        }
    }

    #[test]
    fn surface_tiers_are_distinct() {
        for (name, p) in [("DARK", DARK), ("LIGHT", LIGHT)] {
            let tiers = [p.surface.page, p.surface.card, p.surface.raised];
            for i in 0..tiers.len() {
                for j in (i + 1)..tiers.len() {
                    assert_ne!(
                        tiers[i], tiers[j],
                        "{name}: two surface tiers share a colour, so the container would vanish"
                    );
                }
            }
        }
    }

    #[test]
    fn dark_page_is_darker_than_card() {
        // Layering must read as "card floats above page" in both themes.
        let lum = |c: u32| {
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            r + g + b
        };
        assert!(lum(DARK.surface.page) < lum(DARK.surface.card));
        assert!(lum(DARK.surface.card) < lum(DARK.surface.raised));
        assert!(lum(LIGHT.surface.page) > lum(LIGHT.surface.raised));
    }

    #[test]
    fn semantic_and_danger_colors_are_opaque() {
        // A scrim is the only token allowed to be used with alpha.
        for p in [DARK, LIGHT] {
            assert_eq!(p.text_primary >> 24, 0, "text must be opaque RGB");
            assert_eq!(p.accent >> 24, 0);
            assert_eq!(p.danger >> 24, 0);
        }
    }
}
