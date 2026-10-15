use gpui_kit::*;

// Light Palette (Zen Old Paper)
pub const LIGHT_BG: Hsla = hsla(54.0 / 360.0, 0.22, 0.82, 1.0); // #DDDBC7 Old Paper
pub const LIGHT_BG_HOVER: Hsla = hsla(54.0 / 360.0, 0.22, 0.76, 1.0);
pub const LIGHT_BG_ACTIVE: Hsla = hsla(54.0 / 360.0, 0.22, 0.70, 1.0);
pub const LIGHT_SURFACE: Hsla = hsla(54.0 / 360.0, 0.22, 0.78, 1.0);
pub const LIGHT_INK: Hsla = hsla(0.0, 0.0, 0.0, 1.0); // #000000 Blackov Ink
pub const LIGHT_INK_MUTED: Hsla = hsla(0.0, 0.0, 0.2, 0.65);
pub const LIGHT_INK_FAINT: Hsla = hsla(0.0, 0.0, 0.0, 0.12);
pub const LIGHT_HIGHLIGHT: Hsla = hsla(52.0 / 360.0, 1.0, 0.50, 0.9); // #FFE600

// Dark Palette (Pure Black #000 / Obsidian Ink)
pub const DARK_BG: Hsla = hsla(0.0, 0.0, 0.0, 1.0); // #000000 True Pitch Black
pub const DARK_BG_HOVER: Hsla = hsla(0.0, 0.0, 0.08, 1.0); // #141414
pub const DARK_BG_ACTIVE: Hsla = hsla(0.0, 0.0, 0.14, 1.0); // #242424
pub const DARK_SURFACE: Hsla = hsla(0.0, 0.0, 0.05, 1.0); // #0D0D0D subtle dark elevation
pub const DARK_INK: Hsla = hsla(0.0, 0.0, 0.96, 1.0); // Crisp pure white ink
pub const DARK_INK_MUTED: Hsla = hsla(0.0, 0.0, 0.60, 1.0); // Muted silver
pub const DARK_INK_FAINT: Hsla = hsla(0.0, 0.0, 1.0, 0.14); // Hairline white border
pub const DARK_HIGHLIGHT: Hsla = hsla(48.0 / 360.0, 0.95, 0.48, 0.9); // Warm luminous highlight

pub const AVATAR_ICON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Home,
    Browse,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EditorModeView {
    Visual, // Notion-style WYSIWYG Live Preview mode
    Source, // Styled source-code Editor with syntax highlighting & line numbers
}

pub struct AppTheme {
    pub is_dark: bool,
    pub bg: Hsla,
    pub bg_hover: Hsla,
    pub bg_active: Hsla,
    pub surface: Hsla,
    pub ink: Hsla,
    pub ink_muted: Hsla,
    pub ink_faint: Hsla,
    pub highlight: Hsla,
}

impl AppTheme {
    pub fn new(is_dark: bool) -> Self {
        if is_dark {
            Self {
                is_dark: true,
                bg: DARK_BG,
                bg_hover: DARK_BG_HOVER,
                bg_active: DARK_BG_ACTIVE,
                surface: DARK_SURFACE,
                ink: DARK_INK,
                ink_muted: DARK_INK_MUTED,
                ink_faint: DARK_INK_FAINT,
                highlight: DARK_HIGHLIGHT,
            }
        } else {
            Self {
                is_dark: false,
                bg: LIGHT_BG,
                bg_hover: LIGHT_BG_HOVER,
                bg_active: LIGHT_BG_ACTIVE,
                surface: LIGHT_SURFACE,
                ink: LIGHT_INK,
                ink_muted: LIGHT_INK_MUTED,
                ink_faint: LIGHT_INK_FAINT,
                highlight: LIGHT_HIGHLIGHT,
            }
        }
    }
}
