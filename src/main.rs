mod models;
mod storage;

use std::sync::Arc;
use std::time::Instant;

use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Editor, EditorState, Input, InputEvent, InputState},
    theme::{Theme, ThemeMode},
    Icon, IconName, Sizable, TitleBar,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use models::{ChecklistItem, Note, VoiceRecording};
use storage::Storage;

// Light Palette (Zen Old Paper)
const LIGHT_BG: Hsla = hsla(54.0 / 360.0, 0.22, 0.82, 1.0); // #DDDBC7 Old Paper
const LIGHT_BG_HOVER: Hsla = hsla(54.0 / 360.0, 0.22, 0.76, 1.0);
const LIGHT_BG_ACTIVE: Hsla = hsla(54.0 / 360.0, 0.22, 0.70, 1.0);
const LIGHT_SURFACE: Hsla = hsla(54.0 / 360.0, 0.22, 0.78, 1.0);
const LIGHT_INK: Hsla = hsla(0.0, 0.0, 0.0, 1.0); // #000000 Blackov Ink
const LIGHT_INK_MUTED: Hsla = hsla(0.0, 0.0, 0.2, 0.65);
const LIGHT_INK_FAINT: Hsla = hsla(0.0, 0.0, 0.0, 0.12);
const LIGHT_HIGHLIGHT: Hsla = hsla(52.0 / 360.0, 1.0, 0.50, 0.9); // #FFE600

// Dark Palette (Pure Black #000 / Obsidian Ink)
const DARK_BG: Hsla = hsla(0.0, 0.0, 0.0, 1.0); // #000000 True Pitch Black
const DARK_BG_HOVER: Hsla = hsla(0.0, 0.0, 0.08, 1.0); // #141414
const DARK_BG_ACTIVE: Hsla = hsla(0.0, 0.0, 0.14, 1.0); // #242424
const DARK_SURFACE: Hsla = hsla(0.0, 0.0, 0.05, 1.0); // #0D0D0D subtle dark elevation
const DARK_INK: Hsla = hsla(0.0, 0.0, 0.96, 1.0); // Crisp pure white ink
const DARK_INK_MUTED: Hsla = hsla(0.0, 0.0, 0.60, 1.0); // Muted silver
const DARK_INK_FAINT: Hsla = hsla(0.0, 0.0, 1.0, 0.14); // Hairline white border
const DARK_HIGHLIGHT: Hsla = hsla(48.0 / 360.0, 0.95, 0.48, 0.9); // Warm luminous highlight

const AVATAR_ICON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Home,
    Browse,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub struct NotesApp {
    storage: Storage,
    notes: Vec<Note>,
    selected_id: Option<String>,
    search_query: String,
    current_screen: CurrentScreen,
    is_syncing: bool,
    show_add_modal: bool,
    is_dark_mode: bool,
    editor_mode: EditorModeView,
    zen_focus_mode: bool,

    // Status message for save feedback
    status_message: Option<(String, Instant)>,

    search_state: Entity<InputState>,
    title_state: Entity<InputState>,
    editor_state: Entity<EditorState>,

    // Cached asset data
    lamp_svg_data: Option<Arc<[u8]>>,
    logo_png_path: Option<std::path::PathBuf>,

    // Auth & User Profile State
    show_auth_modal: bool,
    is_signed_in: bool,
    user_name: String,
    user_email: String,

    _subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineTok<'a> {
    Plain(&'a str),
    Bold(&'a str),
    Italic(&'a str),
    Highlight(&'a str),
    Strike(&'a str),
    Code(&'a str),
}

pub fn parse_inline_tokens<'a>(mut text: &'a str) -> Vec<InlineTok<'a>> {
    let mut tokens = Vec::new();

    while !text.is_empty() {
        let bold_pos = text.find("**");
        let highlight_pos = text.find("==");
        let strike_pos = text.find("~~");
        let code_pos = text.find('`');
        let italic_pos = text.find('*');

        let mut candidates: Vec<(usize, &str)> = Vec::new();
        if let Some(p) = bold_pos {
            candidates.push((p, "**"));
        }
        if let Some(p) = highlight_pos {
            candidates.push((p, "=="));
        }
        if let Some(p) = strike_pos {
            candidates.push((p, "~~"));
        }
        if let Some(p) = code_pos {
            candidates.push((p, "`"));
        }
        if let Some(p) = italic_pos {
            if bold_pos != Some(p) {
                candidates.push((p, "*"));
            }
        }

        candidates.sort_by_key(|&(p, _)| p);

        if let Some(&(first_pos, delim)) = candidates.first() {
            if first_pos > 0 {
                tokens.push(InlineTok::Plain(&text[..first_pos]));
            }
            let after_delim = &text[first_pos + delim.len()..];
            if let Some(end_pos) = after_delim.find(delim) {
                let inner = &after_delim[..end_pos];
                match delim {
                    "**" => tokens.push(InlineTok::Bold(inner)),
                    "==" => tokens.push(InlineTok::Highlight(inner)),
                    "~~" => tokens.push(InlineTok::Strike(inner)),
                    "`" => tokens.push(InlineTok::Code(inner)),
                    "*" => tokens.push(InlineTok::Italic(inner)),
                    _ => tokens.push(InlineTok::Plain(inner)),
                }
                text = &after_delim[end_pos + delim.len()..];
            } else {
                tokens.push(InlineTok::Plain(&text[first_pos..first_pos + delim.len()]));
                text = after_delim;
            }
        } else {
            tokens.push(InlineTok::Plain(text));
            break;
        }
    }

    tokens
}

#[derive(Debug, Clone)]
pub enum MarkdownBlock {
    Heading(usize, String),
    Checklist {
        line_idx: usize,
        is_checked: bool,
        text: String,
    },
    Bullet(String),
    Numbered(String, String),
    Quote(String),
    Callout {
        kind: String,
        text: String,
    },
    CodeBlock {
        language: String,
        code: String,
    },
    Divider,
    Paragraph(String),
    Blank,
}

pub fn parse_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // 1. Multi-line Code Block
        if trimmed.starts_with("```") {
            let lang = trimmed.trim_start_matches('`').trim().to_string();
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let code_line = lines[i];
                if code_line.trim().starts_with("```") {
                    i += 1;
                    break;
                }
                code_lines.push(code_line);
                i += 1;
            }
            blocks.push(MarkdownBlock::CodeBlock {
                language: if lang.is_empty() { "code".to_string() } else { lang },
                code: code_lines.join("\n"),
            });
            continue;
        }

        // 2. Blank line
        if trimmed.is_empty() {
            blocks.push(MarkdownBlock::Blank);
            i += 1;
            continue;
        }

        // 3. Horizontal Rule
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(MarkdownBlock::Divider);
            i += 1;
            continue;
        }

        // 4. Headings
        if let Some(h1) = trimmed.strip_prefix("# ") {
            blocks.push(MarkdownBlock::Heading(1, h1.to_string()));
            i += 1;
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            blocks.push(MarkdownBlock::Heading(2, h2.to_string()));
            i += 1;
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            blocks.push(MarkdownBlock::Heading(3, h3.to_string()));
            i += 1;
            continue;
        }

        // 5. Checklist Task: - [ ] or - [x] or - [X]
        if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            let is_checked = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
            let text = trimmed[6..].to_string();
            blocks.push(MarkdownBlock::Checklist {
                line_idx: i,
                is_checked,
                text,
            });
            i += 1;
            continue;
        }

        // 6. Callouts: > [!NOTE], > [!TIP], > [!WARNING], or emojis 💡, ⚠️, ℹ️, 📌
        if trimmed.starts_with("> [!") {
            if let Some(end_bracket) = trimmed.find(']') {
                let kind = trimmed[4..end_bracket].to_string();
                let mut text = trimmed[end_bracket + 1..].trim().to_string();
                i += 1;
                while i < lines.len() && lines[i].trim().starts_with("> ") {
                    let next_trim = lines[i].trim();
                    text.push(' ');
                    text.push_str(&next_trim[2..]);
                    i += 1;
                }
                blocks.push(MarkdownBlock::Callout { kind, text });
                continue;
            }
        }
        if trimmed.starts_with("💡") || trimmed.starts_with("⚠️") || trimmed.starts_with("ℹ️") || trimmed.starts_with("📌") {
            let icon_len = trimmed.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let text = trimmed[icon_len..].trim().to_string();
            blocks.push(MarkdownBlock::Callout {
                kind: "NOTE".to_string(),
                text,
            });
            i += 1;
            continue;
        }

        // 7. Blockquote
        if let Some(quote_text) = trimmed.strip_prefix("> ") {
            let mut text = quote_text.to_string();
            i += 1;
            while i < lines.len() && lines[i].trim().starts_with("> ") {
                let next_trim = lines[i].trim();
                text.push(' ');
                text.push_str(&next_trim[2..]);
                i += 1;
            }
            blocks.push(MarkdownBlock::Quote(text));
            continue;
        }

        // 8. Bullet List
        if trimmed.starts_with("• ") || trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = trimmed[2..].trim().to_string();
            blocks.push(MarkdownBlock::Bullet(text));
            i += 1;
            continue;
        }

        // 9. Numbered List: e.g. "1. " or "2. "
        if let Some(dot_idx) = trimmed.find(". ") {
            if dot_idx > 0 && dot_idx <= 4 && trimmed[..dot_idx].chars().all(|c| c.is_ascii_digit()) {
                let num = trimmed[..=dot_idx].to_string();
                let text = trimmed[dot_idx + 2..].trim().to_string();
                blocks.push(MarkdownBlock::Numbered(num, text));
                i += 1;
                continue;
            }
        }

        // 10. Normal Paragraph
        blocks.push(MarkdownBlock::Paragraph(trimmed.to_string()));
        i += 1;
    }

    blocks
}

impl NotesApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let storage = Storage::new();
        let notes = storage.load_all();
        let selected_id = notes.first().map(|n| n.id.clone());

        let initial_title = notes.first().map(|n| n.title.clone()).unwrap_or_default();
        let initial_body = notes.first().map(|n| n.body.clone()).unwrap_or_default();

        let search_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search notes...")
        });

        let title_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Title...")
        });

        let editor_state = cx.new(|cx| {
            EditorState::new(window, cx)
                .placeholder("Start writing or use Markdown formatting (## Heading, - [ ] Checklist, > Quote)...")
                .language("markdown")
                .line_number(true)
                .folding(true)
        });

        title_state.update(cx, |s, cx| {
            s.set_value(&initial_title, window, cx);
        });
        editor_state.update(cx, |s, cx| {
            s.set_value(&initial_body, window, cx);
        });

        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe_in(&search_state, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.search_query = state.read(cx).value().to_string().to_lowercase();
                cx.notify();
            }
        }));

        subscriptions.push(cx.subscribe_in(&title_state, window, |this, state, event, _, cx| {
            if this.is_syncing {
                return;
            }
            if matches!(event, InputEvent::Change) {
                let new_title = state.read(cx).value().to_string();
                if let Some(id) = &this.selected_id {
                    if let Some(note) = this.notes.iter_mut().find(|n| n.id == *id) {
                        note.title = new_title;
                        note.updated_at = chrono::Local::now();
                        let _ = this.storage.save(note);
                        cx.notify();
                    }
                }
            }
        }));

        subscriptions.push(cx.subscribe_in(&editor_state, window, |this, state, event, _, cx| {
            if this.is_syncing {
                return;
            }
            if matches!(event, InputEvent::Change) {
                let new_body = state.read(cx).value().to_string();
                if let Some(id) = &this.selected_id {
                    if let Some(note) = this.notes.iter_mut().find(|n| n.id == *id) {
                        note.body = new_body;
                        note.updated_at = chrono::Local::now();
                        let _ = this.storage.save(note);
                        cx.notify();
                    }
                }
            }
        }));

        // Load assets safely
        let lamp_svg_data = std::fs::read("assests/zen_lamp.svg")
            .ok()
            .map(Arc::from);

        let logo_path = std::path::Path::new("assests/zen_note_logo.png");
        let logo_png_path = if logo_path.exists() {
            std::fs::canonicalize(logo_path).ok()
        } else {
            None
        };

        Self {
            storage,
            notes,
            selected_id,
            search_query: String::new(),
            current_screen: CurrentScreen::Home,
            is_syncing: false,
            show_add_modal: false,
            is_dark_mode: false,
            editor_mode: EditorModeView::Visual,
            zen_focus_mode: false,
            status_message: None,
            search_state,
            title_state,
            editor_state,
            lamp_svg_data,
            logo_png_path,
            show_auth_modal: false,
            is_signed_in: false,
            user_name: "Nick M.".to_string(),
            user_email: "nick@notty.app".to_string(),
            _subscriptions: subscriptions,
        }
    }

    pub fn theme(&self) -> AppTheme {
        AppTheme::new(self.is_dark_mode)
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.is_dark_mode = !self.is_dark_mode;
        let mode = if self.is_dark_mode {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        Theme::change(mode, None, cx);
        cx.notify();
    }

    pub fn open_note(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_id = Some(id.to_string());
        if let Some(note) = self.notes.iter().find(|n| n.id == id).cloned() {
            self.is_syncing = true;
            self.title_state.update(cx, |s, cx| {
                s.set_value(&note.title, window, cx);
            });
            self.editor_state.update(cx, |s, cx| {
                s.set_value(&note.body, window, cx);
            });
            self.is_syncing = false;
        }
        self.current_screen = CurrentScreen::Editor;
        cx.notify();
    }

    pub fn create_new_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_note = Note::new("", "");
        let _ = self.storage.save(&new_note);
        let id = new_note.id.clone();
        self.notes.insert(0, new_note);

        self.selected_id = Some(id);
        self.is_syncing = true;
        self.title_state.update(cx, |s, cx| {
            s.set_value("", window, cx);
        });
        self.editor_state.update(cx, |s, cx| {
            s.set_value("", window, cx);
        });
        self.is_syncing = false;
        self.current_screen = CurrentScreen::Editor;
        cx.notify();
    }

    pub fn delete_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_id.clone() {
            let _ = self.storage.delete(&id);
            self.notes.retain(|n| n.id != id);

            if let Some(first) = self.notes.first().cloned() {
                self.open_note(&first.id, window, cx);
            } else {
                self.selected_id = None;
                self.current_screen = CurrentScreen::Home;
            }
            cx.notify();
        }
    }

    pub fn toggle_checklist_item(&mut self, item_id: &str, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                if let Some(item) = note.checklist.iter_mut().find(|i| i.id == item_id) {
                    item.checked = !item.checked;
                    note.updated_at = chrono::Local::now();
                    let _ = this_save(note, &self.storage);
                    cx.notify();
                }
            }
        }
    }

    pub fn toggle_markdown_checkbox(
        &mut self,
        target_line_idx: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let lines: Vec<&str> = note.body.lines().collect();
                let mut new_lines = Vec::with_capacity(lines.len());
                for (idx, line) in lines.iter().enumerate() {
                    if idx == target_line_idx {
                        let trimmed = line.trim_start();
                        let indent = &line[..(line.len() - trimmed.len())];
                        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                            new_lines.push(format!("{}- [x] {}", indent, rest));
                        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
                            new_lines.push(format!("{}- [ ] {}", indent, rest));
                        } else if let Some(rest) = trimmed.strip_prefix("- [X] ") {
                            new_lines.push(format!("{}- [ ] {}", indent, rest));
                        } else {
                            new_lines.push(line.to_string());
                        }
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                note.body = new_lines.join("\n");
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.body.clone();
                self.is_syncing = true;
                self.editor_state.update(cx, |s, cx| {
                    s.set_value(&body_str, window, cx);
                });
                self.is_syncing = false;
                cx.notify();
            }
        }
    }

    pub fn add_checklist_item(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let count = note.checklist.len() + 1;
                note.checklist.push(ChecklistItem {
                    id: format!("chk-{}", uuid::Uuid::new_v4()),
                    text: format!("Task item {}", count),
                    checked: false,
                });
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn remove_checklist_item(&mut self, item_id: &str, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.checklist.retain(|i| i.id != item_id);
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn toggle_voice_player(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                if let Some(voice) = &mut note.voice {
                    voice.is_playing = !voice.is_playing;
                    note.updated_at = chrono::Local::now();
                    let _ = this_save(note, &self.storage);
                    cx.notify();
                }
            }
        }
    }

    pub fn remove_voice_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.voice = None;
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn add_sample_voice_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.voice = Some(VoiceRecording {
                    id: format!("rec-{}", uuid::Uuid::new_v4()),
                    duration_seconds: 48,
                    current_seconds: 14,
                    is_playing: false,
                });
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                self.show_add_modal = false;
                cx.notify();
            }
        }
    }

    pub fn add_sample_image(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.images.push("zen_art".to_string());
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                self.show_add_modal = false;
                cx.notify();
            }
        }
    }

    pub fn insert_formatting(&mut self, syntax: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let current_body = &note.body;
                let prepend_newline = !current_body.is_empty() && !current_body.ends_with('\n');
                let prefix = if prepend_newline { "\n" } else { "" };

                match syntax {
                    "h1" => note.body.push_str(&format!("{}# Heading 1\n", prefix)),
                    "h2" => note.body.push_str(&format!("{}## Heading 2\n", prefix)),
                    "h3" => note.body.push_str(&format!("{}### Heading 3\n", prefix)),
                    "bold" => note.body.push_str("**bold text**"),
                    "italic" => note.body.push_str("*italic thought*"),
                    "underline" => note.body.push_str("<u>underlined text</u>"),
                    "highlight" => note.body.push_str("==highlighted note=="),
                    "strike" => note.body.push_str("~~struck text~~"),
                    "code_inline" => note.body.push_str("`code snippet`"),
                    "bullet" => note.body.push_str(&format!("{}- Item point\n", prefix)),
                    "number" => note.body.push_str(&format!("{}1. Step item\n", prefix)),
                    "quote" => note.body.push_str(&format!("{}> Serenity in simplicity.\n", prefix)),
                    "callout" => note.body.push_str(&format!("{}> [!NOTE] Zen is not a destination, it is an approach.\n", prefix)),
                    "code" => note.body.push_str(&format!("{}```rust\nfn main() {{\n    println!(\"Simplicity\");\n}}\n```\n", prefix)),
                    "checklist" => {
                        note.body.push_str(&format!("{}- [ ] New task item\n", prefix));
                    }
                    "divider" => note.body.push_str(&format!("{}---\n", prefix)),
                    _ => {}
                }
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.body.clone();
                self.is_syncing = true;
                self.editor_state.update(cx, |s, cx| {
                    s.set_value(&body_str, window, cx);
                });
                self.is_syncing = false;
                cx.notify();
            }
        }
    }

    pub fn trigger_save(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                self.status_message = Some(("Saved".to_string(), Instant::now()));
                cx.notify();
            }
        }
    }

    fn selected_note(&self) -> Option<&Note> {
        let id = self.selected_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    // Render App Logo: Uses real Zen Logo image if present, with stylized editorial typography
    fn render_logo(&self, t: &AppTheme) -> impl IntoElement {
        let logo_path = self.logo_png_path.clone();

        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                // Logo Icon badge
                if let Some(path) = logo_path {
                    div()
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded_md()
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(t.surface)
                        .border_1()
                        .border_color(t.ink_faint)
                        .child(
                            img(path)
                                .w(px(28.0))
                                .h(px(28.0)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded_md()
                        .bg(t.ink)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_xs()
                        .font_bold()
                        .text_color(t.bg)
                        .child("N")
                        .into_any_element()
                },
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_base()
                            .font_bold()
                            .text_color(t.ink)
                            .child("Notty"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(t.ink_muted)
                            .child("Serene Writing"),
                    ),
            )
    }

    // Render Lamp / Bulb illustration
    fn render_zen_lamp(&self, t: &AppTheme) -> impl IntoElement {
        if let Some(svg_data) = &self.lamp_svg_data {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w(px(220.0))
                .h(px(280.0))
                .child(
                    svg()
                        .data(svg_data)
                        .w(px(200.0))
                        .h(px(260.0))
                        .text_color(t.ink),
                )
                .into_any_element()
        } else {
            // High fidelity editorial lamp fallback
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    div().w(px(2.0)).h(px(80.0)).bg(t.ink),
                )
                .child(
                    div()
                        .w(px(18.0))
                        .h(px(16.0))
                        .rounded_t_sm()
                        .bg(t.ink),
                )
                .child(
                    div()
                        .w(px(72.0))
                        .h(px(88.0))
                        .rounded_t_full()
                        .rounded_b_full()
                        .bg(if t.is_dark {
                            hsla(45.0 / 360.0, 0.45, 0.25, 1.0)
                        } else {
                            hsla(45.0 / 360.0, 0.35, 0.88, 1.0)
                        })
                        .border_2()
                        .border_color(t.ink)
                        .shadow_lg()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(24.0))
                                .h(px(36.0))
                                .rounded_full()
                                .bg(if t.is_dark {
                                    hsla(48.0 / 360.0, 0.90, 0.65, 0.9)
                                } else {
                                    hsla(48.0 / 360.0, 0.60, 0.95, 0.9)
                                }),
                        ),
                )
                .into_any_element()
        }
    }

    // Vertical Japanese-style editorial tagline "Be Minimalistic"
    fn render_vertical_tagline(&self, t: &AppTheme) -> impl IntoElement {
        let word1 = ["B", "E"];
        let word2 = [
            "M", "I", "N", "I", "M", "A", "L", "I", "S", "T", "I", "C",
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .w(px(32.0))
            .py_2()
            // Top accent hairline
            .child(
                div()
                    .w(px(1.0))
                    .h(px(18.0))
                    .bg(t.ink_faint)
                    .mb_2(),
            )
            // Word 1: BE
            .children(word1.into_iter().map(|ch| {
                div()
                    .text_xs()
                    .font_bold()
                    .text_color(t.ink_muted)
                    .line_height(px(15.0))
                    .child(ch)
            }))
            // Delicate gold accent divider
            .child(
                div()
                    .my_2()
                    .w(px(3.0))
                    .h(px(3.0))
                    .rounded_full()
                    .bg(t.highlight),
            )
            // Word 2: MINIMALISTIC
            .children(word2.into_iter().map(|ch| {
                div()
                    .text_xs()
                    .font_bold()
                    .text_color(t.ink_muted)
                    .line_height(px(15.0))
                    .child(ch)
            }))
            // Bottom accent hairline
            .child(
                div()
                    .w(px(1.0))
                    .h(px(18.0))
                    .bg(t.ink_faint)
                    .mt_2(),
            )
    }

    // Avatar Sign In / Profile Button
    fn render_avatar_button(&self, t: &AppTheme, cx: &mut Context<Self>) -> impl IntoElement {
        let is_signed = self.is_signed_in;
        let initials = if is_signed {
            let parts: Vec<&str> = self.user_name.split_whitespace().collect();
            if parts.len() >= 2 {
                format!(
                    "{}{}",
                    parts[0].chars().next().unwrap_or('N'),
                    parts[1].chars().next().unwrap_or('M')
                )
            } else {
                self.user_name.chars().take(2).collect::<String>().to_uppercase()
            }
        } else {
            "Sign In".to_string()
        };

        div()
            .id("btn-avatar-auth")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_full()
            .border_1()
            .border_color(if is_signed { t.ink } else { t.ink_faint })
            .bg(if is_signed { t.surface } else { t.bg })
            .hover(move |s| s.bg(t.bg_hover).border_color(t.ink))
            .active(move |s| s.bg(t.bg_active))
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_auth_modal = true;
                cx.notify();
            }))
            // Avatar circle badge
            .child(
                div()
                    .w(px(26.0))
                    .h(px(26.0))
                    .rounded_full()
                    .bg(if is_signed { t.ink } else { t.surface })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(if is_signed { t.bg } else { t.ink })
                    .text_xs()
                    .font_bold()
                    .relative()
                    .child(if is_signed {
                        div().child(initials)
                    } else {
                        div()
                            .w(px(14.0))
                            .h(px(14.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                svg()
                                    .data(AVATAR_ICON_SVG.as_bytes())
                                    .w(px(13.0))
                                    .h(px(13.0))
                                    .text_color(t.ink),
                            )
                    })
                    // Online / Synced status dot
                    .child(
                        div()
                            .absolute()
                            .right(px(-1.0))
                            .bottom(px(-1.0))
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .border_1()
                            .border_color(t.bg)
                            .bg(if is_signed {
                                hsla(140.0 / 360.0, 0.80, 0.45, 1.0)
                            } else {
                                t.ink_muted
                            }),
                    ),
            )
            // Label next to avatar
            .child(
                div()
                    .text_xs()
                    .font_medium()
                    .text_color(t.ink)
                    .pr_1()
                    .child(if is_signed {
                        self.user_name.clone()
                    } else {
                        "Sign In".to_string()
                    }),
            )
    }

    // Modal dialog for Sign In & Profile Management
    fn render_auth_modal(&self, t: &AppTheme, cx: &mut Context<Self>) -> impl IntoElement {
        let is_signed = self.is_signed_in;
        let user_name = self.user_name.clone();
        let user_email = self.user_email.clone();

        div()
            .id("auth-modal-backdrop")
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.75))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_auth_modal = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("auth-modal-card")
                    .w(px(380.0))
                    .p_6()
                    .rounded_lg()
                    .border_2()
                    .border_color(if t.is_dark { t.ink_faint } else { t.ink })
                    .bg(t.bg)
                    .text_color(t.ink)
                    .shadow_2xl()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                    })
                    // Modal Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .h(px(24.0))
                                            .rounded_sm()
                                            .bg(t.ink)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_xs()
                                            .font_bold()
                                            .text_color(t.bg)
                                            .child("N"),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child(if is_signed {
                                                "Account Profile"
                                            } else {
                                                "Sign in to Notty"
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-auth-modal")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded_sm()
                                    .text_color(t.ink_muted)
                                    .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Close).size_4()),
                            ),
                    )
                    // Modal Body
                    .child(if is_signed {
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_4()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        div()
                                            .w(px(46.0))
                                            .h(px(46.0))
                                            .rounded_full()
                                            .bg(t.ink)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.bg)
                                            .child("NM"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_bold()
                                                    .text_color(t.ink)
                                                    .child(user_name),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(t.ink_muted)
                                                    .child(user_email),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(hsla(140.0 / 360.0, 0.70, 0.45, 1.0))
                                    .child(
                                        div()
                                            .w(px(8.0))
                                            .h(px(8.0))
                                            .rounded_full()
                                            .bg(hsla(140.0 / 360.0, 0.80, 0.45, 1.0)),
                                    )
                                    .child("Cloud Sync: Active & Synced"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(t.ink_muted)
                                    .child("All your notes, checklists, and voice memos are backed up to your Notty account."),
                            )
                            .child(
                                div()
                                    .id("btn-auth-sign-out")
                                    .cursor_pointer()
                                    .py_2()
                                    .border_1()
                                    .border_color(t.ink)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = false;
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child("Sign Out"),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink_muted)
                                    .child("Sign in to automatically sync your notes across devices and unlock encrypted cloud backup."),
                            )
                            .child(
                                div()
                                    .id("btn-auth-sign-in-nick")
                                    .cursor_pointer()
                                    .p_3()
                                    .rounded_md()
                                    .bg(t.ink)
                                    .text_color(t.bg)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = true;
                                        this.user_name = "Nick M.".to_string();
                                        this.user_email = "nick@notty.app".to_string();
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .w(px(30.0))
                                                    .h(px(30.0))
                                                    .rounded_full()
                                                    .bg(t.bg)
                                                    .text_color(t.ink)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("NM"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("Sign in as Nick M."),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .opacity(0.7)
                                                            .child("nick@notty.app"),
                                                    ),
                                            ),
                                    )
                                    .child(Icon::new(IconName::ChevronRight).size_4().text_color(t.bg)),
                            )
                            .child(
                                div()
                                    .id("btn-auth-quick-guest")
                                    .cursor_pointer()
                                    .py_2()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(t.ink)
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = true;
                                        this.user_name = "Zen Writer".to_string();
                                        this.user_email = "writer@notty.app".to_string();
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child("Sign in as Zen Writer"),
                            )
                            .into_any_element()
                    }),
            )
    }

    // Home Screen View
    fn render_home(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            // Top Nav
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .pt_6()
                    .child(self.render_logo(&t))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            // Dark / Light Mode Toggle Button
                            .child(
                                Button::new("btn-home-theme-toggle")
                                    .icon(if t.is_dark {
                                        IconName::Sun
                                    } else {
                                        IconName::Moon
                                    })
                                    .ghost()
                                    .tooltip(if t.is_dark {
                                        "Switch to Light Mode (Old Paper)"
                                    } else {
                                        "Switch to Dark Mode (Pitch Black #000)"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_theme(cx);
                                    })),
                            )
                            // Search Button
                            .child(
                                Button::new("btn-home-search")
                                    .icon(IconName::Search)
                                    .ghost()
                                    .tooltip("Browse & Search Notes")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.current_screen = CurrentScreen::Browse;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Center Illustration & Vertical Tagline on the side
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .relative()
                    // Vertical Japanese-style tagline docked on the side
                    .child(
                        div()
                            .absolute()
                            .left(px(36.0))
                            .child(self.render_vertical_tagline(&t)),
                    )
                    // Centered Zen Lamp
                    .child(self.render_zen_lamp(&t)),
            )
            // Bottom Action Area & Avatar Sign-In Experience
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .pb_8()
                    .child(
                        // Boxed [ New Note ] Button
                        div()
                            .id("btn-cover-new-note")
                            .cursor_pointer()
                            .px_10()
                            .py_3()
                            .border_2()
                            .border_color(t.ink)
                            .bg(t.bg)
                            .hover(move |s| s.bg(t.bg_hover))
                            .active(move |s| s.bg(t.bg_active))
                            .text_sm()
                            .font_bold()
                            .text_color(t.ink)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_new_note(window, cx);
                            }))
                            .child("New Note"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .italic()
                            .text_color(t.ink_muted)
                            .child("or"),
                    )
                    .child(
                        div()
                            .id("btn-cover-browse-notes")
                            .cursor_pointer()
                            .text_sm()
                            .font_bold()
                            .text_color(t.ink)
                            .hover(|s| s.underline())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_screen = CurrentScreen::Browse;
                                cx.notify();
                            }))
                            .child("Browse Notes"),
                    )
                    // Avatar Sign-In Experience at the bottom
                    .child(
                        div()
                            .mt_3()
                            .child(self.render_avatar_button(&t, cx)),
                    ),
            )
    }

    // Browse & Search Screen View
    fn render_browse(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();

        let filtered_notes: Vec<Note> = self
            .notes
            .iter()
            .filter(|n| {
                if self.search_query.is_empty() {
                    true
                } else {
                    n.title.to_lowercase().contains(&self.search_query)
                        || n.body.to_lowercase().contains(&self.search_query)
                }
            })
            .cloned()
            .collect();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            // Navigation bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_4()
                    .border_b_1()
                    .border_color(t.ink_faint)
                    .child(
                        Button::new("btn-browse-back")
                            .icon(IconName::ChevronLeft)
                            .ghost()
                            .tooltip("Home")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_screen = CurrentScreen::Home;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                if let Some(path) = &self.logo_png_path {
                                    div()
                                        .w(px(20.0))
                                        .h(px(20.0))
                                        .rounded_sm()
                                        .overflow_hidden()
                                        .child(
                                            img(path.clone())
                                                .w(px(20.0))
                                                .h(px(20.0)),
                                        )
                                        .into_any_element()
                                } else {
                                    div()
                                        .w(px(16.0))
                                        .h(px(16.0))
                                        .rounded_sm()
                                        .bg(t.ink)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(t.bg)
                                        .child("N")
                                        .into_any_element()
                                },
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .child("Browse Notes"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(t.ink_muted)
                                    .child(format!("({})", self.notes.len())),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("btn-browse-theme-toggle")
                                    .icon(if t.is_dark {
                                        IconName::Sun
                                    } else {
                                        IconName::Moon
                                    })
                                    .ghost()
                                    .tooltip(if t.is_dark { "Light mode" } else { "Dark mode" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_theme(cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-browse-new-top")
                                    .icon(IconName::Plus)
                                    .ghost()
                                    .tooltip("New Note")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_new_note(window, cx);
                                    })),
                            )
                            .child(self.render_avatar_button(&t, cx)),
                    ),
            )
            // Search Input Row
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_6()
                    .py_3()
                    .border_b_1()
                    .border_color(t.ink_faint)
                    .child(Icon::new(IconName::Search).size_4().text_color(t.ink))
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(&self.search_state).bordered(false).small()),
                    )
                    .when(!self.search_query.is_empty(), |this| {
                        this.child(
                            Button::new("btn-clear-search")
                                .icon(IconName::Close)
                                .ghost()
                                .small()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.search_query.clear();
                                    this.search_state.update(cx, |s, cx| {
                                        s.set_value("", window, cx);
                                    });
                                    cx.notify();
                                })),
                        )
                    }),
            )
            // Note Table List
            .child(
                div()
                    .id("browse-notes-list")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .when(filtered_notes.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_16()
                                .gap_2()
                                .child(Icon::new(IconName::FileText).size_8().text_color(t.ink_muted))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(t.ink_muted)
                                        .child("No notes found"),
                                ),
                        )
                    })
                    .children(filtered_notes.into_iter().map(|note| {
                        let note_id = note.id.clone();
                        let title = note.display_title().to_string();
                        let date = note.formatted_date();
                        let is_active = self.selected_id.as_deref() == Some(&note_id);

                        div()
                            .id(SharedString::from(format!("browse-item-{}", note_id)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_6()
                            .py_4()
                            .border_b_1()
                            .border_color(t.ink_faint)
                            .bg(if is_active { t.surface } else { t.bg })
                            .cursor_pointer()
                            .hover(move |s| s.bg(t.bg_hover))
                            .active(move |s| s.bg(t.bg_active))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_note(&note_id, window, cx);
                            }))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child(title),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(t.ink_muted)
                                            .child(note.snippet()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .when(!note.checklist.is_empty(), |this| {
                                        let done = note.checklist.iter().filter(|i| i.checked).count();
                                        let total = note.checklist.len();
                                        this.child(
                                            div()
                                                .text_xs()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .border_1()
                                                .border_color(t.ink_faint)
                                                .text_color(t.ink_muted)
                                                .child(format!("{}/{}", done, total)),
                                        )
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(t.ink_muted)
                                            .child(date),
                                    ),
                            )
                    })),
            )
            // Bottom New Note button
            .child(
                div()
                    .flex()
                    .justify_center()
                    .py_4()
                    .border_t_1()
                    .border_color(t.ink_faint)
                    .child(
                        div()
                            .id("btn-browse-new-bottom")
                            .cursor_pointer()
                            .px_8()
                            .py_2()
                            .border_2()
                            .border_color(t.ink)
                            .bg(t.bg)
                            .hover(move |s| s.bg(t.bg_hover))
                            .text_xs()
                            .font_bold()
                            .text_color(t.ink)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_new_note(window, cx);
                            }))
                            .child("New Note"),
                    ),
            )
    }

    // Inline formatting tokens: **bold**, *italic*, ==highlight==, ~~strike~~, `code`
    fn render_inline_spans(&self, text: &str, t: &AppTheme) -> impl IntoElement {
        let tokens = parse_inline_tokens(text);
        div()
            .flex()
            .flex_wrap()
            .items_baseline()
            .gap_x_1()
            .gap_y_0p5()
            .children(tokens.into_iter().map(|tok| match tok {
                InlineTok::Plain(s) => div().text_color(t.ink).child(s.to_string()).into_any_element(),
                InlineTok::Bold(s) => div().font_bold().text_color(t.ink).child(s.to_string()).into_any_element(),
                InlineTok::Italic(s) => div().italic().text_color(t.ink).child(s.to_string()).into_any_element(),
                InlineTok::Highlight(s) => div()
                    .bg(t.highlight)
                    .px_1()
                    .py_0p5()
                    .rounded_sm()
                    .text_xs()
                    .font_bold()
                    .text_color(LIGHT_INK)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Strike(s) => div()
                    .line_through()
                    .text_color(t.ink_muted)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Code(s) => div()
                    .font_family(SharedString::from("Consolas, Courier New, monospace"))
                    .text_xs()
                    .bg(t.surface)
                    .border_1()
                    .border_color(t.ink_faint)
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .text_color(t.ink)
                    .child(s.to_string())
                    .into_any_element(),
            }))
    }

    // Notion-style Block Renderer for WYSIWYG Visual Mode
    fn render_notion_blocks(
        &self,
        content: &str,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let blocks = parse_markdown_blocks(content);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .py_2()
            .children(blocks.into_iter().enumerate().map(|(idx, block)| {
                match block {
                    MarkdownBlock::Blank => {
                        div().h(px(8.0)).into_any_element()
                    }
                    MarkdownBlock::Heading(level, text) => {
                        let text_clone = text.clone();
                        match level {
                            1 => div()
                                .mt_4()
                                .mb_2()
                                .text_2xl()
                                .font_bold()
                                .text_color(t.ink)
                                .border_b_1()
                                .border_color(t.ink_faint)
                                .pb_1()
                                .child(self.render_inline_spans(&text_clone, t))
                                .into_any_element(),
                            2 => div()
                                .mt_3()
                                .mb_1p5()
                                .text_xl()
                                .font_bold()
                                .text_color(t.ink)
                                .child(self.render_inline_spans(&text_clone, t))
                                .into_any_element(),
                            _ => div()
                                .mt_2()
                                .mb_1()
                                .text_base()
                                .font_bold()
                                .text_color(t.ink)
                                .child(self.render_inline_spans(&text_clone, t))
                                .into_any_element(),
                        }
                    }
                    MarkdownBlock::Checklist {
                        line_idx,
                        is_checked,
                        text,
                    } => {
                        let is_checked_val = is_checked;
                        let text_copy = text.clone();
                        div()
                            .id(SharedString::from(format!("notion-chk-{}", line_idx)))
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_1()
                            .child(
                                div()
                                    .id(SharedString::from(format!("notion-chk-box-{}", line_idx)))
                                    .cursor_pointer()
                                    .mt_0p5()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .border_1()
                                    .border_color(if is_checked_val { t.ink } else { t.ink_muted })
                                    .bg(if is_checked_val { t.ink } else { t.bg })
                                    .rounded_sm()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(move |s| s.border_color(t.ink).opacity(0.85))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.toggle_markdown_checkbox(line_idx, window, cx);
                                    }))
                                    .when(is_checked_val, |this| {
                                        this.child(
                                            Icon::new(IconName::Check)
                                                .size_3()
                                                .text_color(t.bg),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(if is_checked_val { t.ink_muted } else { t.ink })
                                    .when(is_checked_val, |s| s.line_through())
                                    .child(self.render_inline_spans(&text_copy, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Bullet(text) => {
                        let text_clone = text.clone();
                        div()
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_0p5()
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .mt_2()
                                    .rounded_full()
                                    .bg(t.ink),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(self.render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Numbered(num, text) => {
                        let text_clone = text.clone();
                        div()
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_0p5()
                            .child(
                                div()
                                    .w(px(20.0))
                                    .text_sm()
                                    .font_bold()
                                    .text_color(t.ink_muted)
                                    .child(num),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(self.render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Quote(text) => {
                        let text_clone = text.clone();
                        div()
                            .my_2()
                            .pl_4()
                            .py_1p5()
                            .border_l_4()
                            .border_color(t.ink)
                            .bg(t.surface)
                            .italic()
                            .text_sm()
                            .line_height(relative(1.6))
                            .text_color(t.ink_muted)
                            .child(self.render_inline_spans(&text_clone, t))
                            .into_any_element()
                    }
                    MarkdownBlock::Callout { kind: _, text } => {
                        let text_clone = text.clone();
                        div()
                            .my_2p5()
                            .p_3p5()
                            .rounded_md()
                            .border_1()
                            .border_color(t.highlight)
                            .bg(if t.is_dark {
                                hsla(45.0 / 360.0, 0.40, 0.14, 1.0)
                            } else {
                                hsla(52.0 / 360.0, 0.45, 0.88, 1.0)
                            })
                            .flex()
                            .items_start()
                            .gap_3()
                            .child(
                                div()
                                    .mt_0p5()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded_sm()
                                    .bg(t.highlight)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(Icon::new(IconName::Info).size_3().text_color(LIGHT_INK)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(self.render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::CodeBlock { language, code } => {
                        let code_clone = code.clone();
                        let lang_upper = if language.is_empty() {
                            "CODE".to_string()
                        } else {
                            language.to_uppercase()
                        };

                        div()
                            .my_3()
                            .rounded_md()
                            .border_1()
                            .border_color(t.ink_faint)
                            .bg(if t.is_dark {
                                hsla(0.0, 0.0, 0.06, 1.0)
                            } else {
                                hsla(54.0 / 360.0, 0.18, 0.74, 1.0)
                            })
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_1p5()
                                    .border_b_1()
                                    .border_color(t.ink_faint)
                                    .bg(if t.is_dark {
                                        hsla(0.0, 0.0, 0.09, 1.0)
                                    } else {
                                        hsla(54.0 / 360.0, 0.20, 0.78, 1.0)
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .w(px(6.0))
                                                    .h(px(6.0))
                                                    .rounded_full()
                                                    .bg(t.highlight),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(t.ink_muted)
                                                    .child(lang_upper),
                                            ),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-copy-code-{}", idx)))
                                            .icon(IconName::Copy)
                                            .ghost()
                                            .small()
                                            .label("Copy")
                                            .tooltip("Copy snippet to clipboard")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(code_clone.clone()));
                                                this.status_message = Some(("Code copied!".to_string(), Instant::now()));
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .p_3()
                                    .overflow_hidden()
                                    .font_family(SharedString::from("Consolas, Courier New, monospace"))
                                    .text_xs()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .children(code.lines().enumerate().map(|(line_num, line_str)| {
                                        div()
                                            .flex()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .w(px(24.0))
                                                    .text_right()
                                                    .text_color(t.ink_faint)
                                                    .child((line_num + 1).to_string()),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .child(line_str.to_string()),
                                            )
                                    }))
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Divider => {
                        div()
                            .my_4()
                            .h_px()
                            .w_full()
                            .bg(t.ink_faint)
                            .into_any_element()
                    }
                    MarkdownBlock::Paragraph(text) => {
                        let text_clone = text.clone();
                        div()
                            .my_1()
                            .text_sm()
                            .line_height(relative(1.6))
                            .text_color(t.ink)
                            .child(self.render_inline_spans(&text_clone, t))
                            .into_any_element()
                    }
                }
            }))
    }

    // Notion-style Empty Note Starter Blocks
    fn render_notion_empty_starter(&self, t: &AppTheme, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .py_12()
            .px_6()
            .border_1()
            .border_dashed()
            .border_color(t.ink_faint)
            .rounded_lg()
            .bg(t.surface)
            .child(
                div()
                    .text_base()
                    .font_bold()
                    .text_color(t.ink)
                    .mb_1()
                    .child("Notty — Live Visual Canvas"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(t.ink_muted)
                    .mb_5()
                    .child("Type freely, switch to Source mode, or pick a Notion-style block to begin:"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_center()
                    .gap_2p5()
                    .child(
                        Button::new("starter-checklist")
                            .icon(IconName::Check)
                            .ghost()
                            .small()
                            .label("+ Checklist Task")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("checklist", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-heading")
                            .ghost()
                            .small()
                            .label("+ Heading")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("h1", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-code")
                            .icon(IconName::Copy)
                            .ghost()
                            .small()
                            .label("+ Code Snippet")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("code", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-quote")
                            .ghost()
                            .small()
                            .label("+ Quote")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("quote", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-callout")
                            .icon(IconName::Info)
                            .ghost()
                            .small()
                            .label("+ Callout Note")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("callout", window, cx);
                            })),
                    ),
            )
    }

    // Streamlined Single-Layout Note Editor View (WYSIWYG Live Preview / Visual Mode like Notion)
    fn render_editor(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let active_note = self.selected_note().cloned();

        // Calculate word & character metrics for editor status bar
        let body_content = active_note.as_ref().map(|n| n.body.as_str()).unwrap_or("");
        let word_count = body_content.split_whitespace().count();
        let char_count = body_content.chars().count();
        let read_time_mins = (word_count / 200).max(1);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            .relative()
            // Top Nav: [< Back] [Segmented Mode: Visual | Source] [Zen Mode] [Theme] [Save] [Delete]
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_3()
                    .border_b_1()
                    .border_color(t.ink_faint)
                    .child(
                        Button::new("btn-editor-back")
                            .icon(IconName::ChevronLeft)
                            .ghost()
                            .tooltip("Back to Notes (Esc)")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_screen = CurrentScreen::Browse;
                                cx.notify();
                            })),
                    )
                    // Segmented Editor Mode Switcher (Single Layout: Visual vs Source)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .p_0p5()
                            .rounded_md()
                            .bg(t.surface)
                            .border_1()
                            .border_color(t.ink_faint)
                            // Visual Mode Pill
                            .child(
                                div()
                                    .id("tab-visual")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .font_bold()
                                    .bg(if self.editor_mode == EditorModeView::Visual {
                                        t.bg
                                    } else {
                                        hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .text_color(if self.editor_mode == EditorModeView::Visual {
                                        t.ink
                                    } else {
                                        t.ink_muted
                                    })
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor_mode = EditorModeView::Visual;
                                        cx.notify();
                                    }))
                                    .child("👁 Visual"),
                            )
                            // Source Mode Pill (uses Editor)
                            .child(
                                div()
                                    .id("tab-source")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .font_bold()
                                    .bg(if self.editor_mode == EditorModeView::Source {
                                        t.bg
                                    } else {
                                        hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .text_color(if self.editor_mode == EditorModeView::Source {
                                        t.ink
                                    } else {
                                        t.ink_muted
                                    })
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor_mode = EditorModeView::Source;
                                        cx.notify();
                                    }))
                                    .child("✎ Source"),
                            ),
                    )
                    // Right Toolbar Controls
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            // Zen Focus Mode Toggle
                            .child(
                                Button::new("btn-editor-zen")
                                    .icon(if self.zen_focus_mode {
                                        IconName::Maximize
                                    } else {
                                        IconName::Minimize
                                    })
                                    .ghost()
                                    .small()
                                    .tooltip(if self.zen_focus_mode {
                                        "Exit Zen Focus"
                                    } else {
                                        "Zen Focus Mode"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.zen_focus_mode = !this.zen_focus_mode;
                                        cx.notify();
                                    })),
                            )
                            // Light / Dark Mode Toggle
                            .child(
                                Button::new("btn-editor-theme-toggle")
                                    .icon(if t.is_dark {
                                        IconName::Sun
                                    } else {
                                        IconName::Moon
                                    })
                                    .ghost()
                                    .small()
                                    .tooltip(if t.is_dark { "Light mode" } else { "Dark mode" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_theme(cx);
                                    })),
                            )
                            // Add Media Button
                            .child(
                                div()
                                    .id("btn-editor-add")
                                    .cursor_pointer()
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .hover(|s| s.underline())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_add_modal = !this.show_add_modal;
                                        cx.notify();
                                    }))
                                    .child("Attach"),
                            )
                            // Save Button
                            .child(
                                Button::new("btn-editor-save")
                                    .icon(IconName::Check)
                                    .ghost()
                                    .small()
                                    .tooltip("Save Note")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.trigger_save(cx);
                                    })),
                            )
                            // Delete Button
                            .child(
                                Button::new("btn-editor-delete")
                                    .icon(IconName::Delete)
                                    .ghost()
                                    .small()
                                    .tooltip("Delete Note")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.delete_selected_note(window, cx);
                                    })),
                            ),
                    ),
            )
            // Notion-Style Quick Formatting Strip (hides in Zen focus mode)
            .when(!self.zen_focus_mode, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_6()
                        .py_1p5()
                        .border_b_1()
                        .border_color(t.ink_faint)
                        .bg(t.surface)
                        .child(
                            Button::new("fmt-h1")
                                .ghost()
                                .small()
                                .label("H1")
                                .tooltip("Heading 1 (#)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("h1", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-h2")
                                .ghost()
                                .small()
                                .label("H2")
                                .tooltip("Heading 2 (##)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("h2", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-h3")
                                .ghost()
                                .small()
                                .label("H3")
                                .tooltip("Heading 3 (###)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("h3", window, cx);
                                })),
                        )
                        .child(div().w_px().h(px(14.0)).bg(t.ink_faint).mx_1())
                        .child(
                            Button::new("fmt-bold")
                                .ghost()
                                .small()
                                .label("B")
                                .tooltip("Bold (**text**)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("bold", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-italic")
                                .ghost()
                                .small()
                                .label("I")
                                .tooltip("Italic (*text*)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("italic", window, cx);
                                })),
                        )
                        .child(
                            div()
                                .id("fmt-highlight")
                                .cursor_pointer()
                                .w(px(20.0))
                                .h(px(20.0))
                                .bg(t.highlight)
                                .rounded_sm()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_xs()
                                .font_bold()
                                .text_color(LIGHT_INK)
                                .hover(|s| s.opacity(0.8))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("highlight", window, cx);
                                }))
                                .child("T"),
                        )
                        .child(
                            Button::new("fmt-strike")
                                .ghost()
                                .small()
                                .label("~")
                                .tooltip("Strikethrough (~~text~~)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("strike", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-code-inline")
                                .ghost()
                                .small()
                                .label("`")
                                .tooltip("Inline Code (`code`)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("code_inline", window, cx);
                                })),
                        )
                        .child(div().w_px().h(px(14.0)).bg(t.ink_faint).mx_1())
                        .child(
                            Button::new("fmt-task")
                                .icon(IconName::Check)
                                .ghost()
                                .small()
                                .tooltip("Interactive Task (- [ ])")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("checklist", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-bullet")
                                .icon(IconName::Menu)
                                .ghost()
                                .small()
                                .tooltip("Bullet List (- item)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("bullet", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-number")
                                .ghost()
                                .small()
                                .label("1.")
                                .tooltip("Numbered List (1. item)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("number", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-quote")
                                .ghost()
                                .small()
                                .label("\"")
                                .tooltip("Blockquote (> quote)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("quote", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-callout")
                                .icon(IconName::Info)
                                .ghost()
                                .small()
                                .tooltip("Callout Box (> [!NOTE])")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("callout", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-code-block")
                                .icon(IconName::Copy)
                                .ghost()
                                .small()
                                .tooltip("Code Block (```lang)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("code", window, cx);
                                })),
                        )
                        .child(
                            Button::new("fmt-divider")
                                .ghost()
                                .small()
                                .label("—")
                                .tooltip("Divider (---)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.insert_formatting("divider", window, cx);
                                })),
                        ),
                )
            })
            // Single Document Canvas (WYSIWYG Live Canvas or Source Editor)
            .child(
                div()
                    .id("editor-canvas-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .when(!self.zen_focus_mode, |this| this.px_10().py_6())
                    .when(self.zen_focus_mode, |this| this.px_20().py_8())
                    // Date & Status Header
                    .when(active_note.is_some(), |this| {
                        let note = active_note.as_ref().unwrap();
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .mb_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(t.ink_muted)
                                                .child(note.formatted_full_date()),
                                        )
                                        .child(div().w_px().h(px(10.0)).bg(t.ink_faint))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(t.ink_muted)
                                                .child(format!("{} min read", read_time_mins)),
                                        ),
                                )
                                .when(self.status_message.is_some(), |this| {
                                    let (msg, _) = self.status_message.as_ref().unwrap();
                                    this.child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_sm()
                                            .bg(t.highlight)
                                            .text_xs()
                                            .font_bold()
                                            .text_color(LIGHT_INK)
                                            .child(format!("✓ {}", msg)),
                                    )
                                }),
                        )
                    })
                    // Title row using Input (Single-line value control)
                    .child(
                        div()
                            .mb_2()
                            .child(
                                Input::new(&self.title_state)
                                    .bordered(false)
                                    .large(),
                            ),
                    )
                    // Ruled horizontal line under title
                    .child(
                        div()
                            .h_px()
                            .bg(t.ink)
                            .mb_4(),
                    )
                    // Structured Tasks Section (if note has checklist items)
                    .when(active_note.as_ref().map_or(false, |n| !n.checklist.is_empty()), |this| {
                        let note = active_note.as_ref().unwrap();
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .mb_4()
                                .p_3()
                                .rounded_md()
                                .bg(t.surface)
                                .border_1()
                                .border_color(t.ink_faint)
                                .children(note.checklist.iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let is_checked = item.checked;
                                    let text = item.text.clone();
                                    let del_id = item_id.clone();

                                    div()
                                        .id(SharedString::from(format!("chk-row-{}", item_id)))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap_3()
                                        .child(
                                            div()
                                                .id(SharedString::from(format!("chk-toggle-{}", item_id)))
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .cursor_pointer()
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.toggle_checklist_item(&item_id, cx);
                                                }))
                                                .child(
                                                    div()
                                                        .w(px(16.0))
                                                        .h(px(16.0))
                                                        .border_1()
                                                        .border_color(t.ink)
                                                        .rounded_sm()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .when(is_checked, |this| {
                                                            this.child(
                                                                Icon::new(IconName::Check)
                                                                    .size_3()
                                                                    .text_color(t.ink),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(t.ink)
                                                        .when(is_checked, |this| {
                                                            this.text_color(t.ink_muted).line_through()
                                                        })
                                                        .child(text),
                                                ),
                                        )
                                        .child(
                                            Button::new(SharedString::from(format!("del-chk-{}", del_id)))
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .tooltip("Remove task")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.remove_checklist_item(&del_id, cx);
                                                })),
                                        )
                                })),
                        )
                    })
                    // Single Layout Document Workspace: WYSIWYG Visual Mode OR Source Editor Mode
                    .child(
                        match self.editor_mode {
                            EditorModeView::Visual => {
                                if body_content.trim().is_empty() {
                                    self.render_notion_empty_starter(&t, cx).into_any_element()
                                } else {
                                    div()
                                        .flex_1()
                                        .mb_4()
                                        .child(self.render_notion_blocks(body_content, &t, cx))
                                        .into_any_element()
                                }
                            }
                            EditorModeView::Source => {
                                div()
                                    .flex_1()
                                    .mb_4()
                                    .min_h(px(380.0))
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        Editor::new(&self.editor_state)
                                            .bordered(false)
                                            .h(px(380.0)),
                                    )
                                    .into_any_element()
                            }
                        },
                    )
                    // Visual Mode Bottom Action Strip: Quick adders and switch to source
                    .when(self.editor_mode == EditorModeView::Visual && !body_content.trim().is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pt_4()
                                .pb_2()
                                .border_t_1()
                                .border_color(t.ink_faint)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Button::new("btn-bottom-add-task")
                                                .ghost()
                                                .small()
                                                .icon(IconName::Check)
                                                .label("+ Task")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("checklist", window, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-bottom-add-heading")
                                                .ghost()
                                                .small()
                                                .label("+ Heading")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("h2", window, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-bottom-add-code")
                                                .ghost()
                                                .small()
                                                .icon(IconName::Copy)
                                                .label("+ Code")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("code", window, cx);
                                                })),
                                        ),
                                )
                                .child(
                                    Button::new("btn-switch-to-source")
                                        .ghost()
                                        .small()
                                        .label("Edit Source ✎")
                                        .tooltip("Switch to raw Markdown Editor")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.editor_mode = EditorModeView::Source;
                                            cx.notify();
                                        })),
                                ),
                        )
                    })
                    // Attached images section
                    .when(active_note.as_ref().map_or(false, |n| !n.images.is_empty()), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .my_4()
                                .p_3()
                                .border_1()
                                .border_color(t.ink_faint)
                                .bg(t.surface)
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(140.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(t.bg)
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .gap_2()
                                                .child(Icon::new(IconName::FileText).size_8().text_color(t.ink))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .italic()
                                                        .text_color(t.ink_muted)
                                                        .child("Zen Photo Attachment"),
                                                ),
                                        ),
                                ),
                        )
                    })
                    // Voice player bar (as in inspo wireframes)
                    .when(active_note.as_ref().map_or(false, |n| n.voice.is_some()), |this| {
                        let voice = active_note.as_ref().unwrap().voice.as_ref().unwrap();
                        let is_playing = voice.is_playing;

                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_2()
                                .my_3()
                                .border_1()
                                .border_color(t.ink)
                                .bg(t.bg)
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .child(format!("00:00:{:02}", voice.current_seconds)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .mx_4()
                                        .h(px(2.0))
                                        .bg(t.ink)
                                        .relative()
                                        .child(
                                            div()
                                                .absolute()
                                                .left(px(40.0))
                                                .top(px(-3.0))
                                                .w(px(8.0))
                                                .h(px(8.0))
                                                .rounded_full()
                                                .bg(t.ink),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Button::new("btn-voice-play-pause")
                                                .icon(if is_playing {
                                                    IconName::Play
                                                } else {
                                                    IconName::ChevronRight
                                                })
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.toggle_voice_player(cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-voice-close")
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.remove_voice_recording(cx);
                                                })),
                                        ),
                                ),
                        )
                    }),
            )
            // Streamlined Editor Status Footer: Word count, character count, and quick task adder
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_2()
                    .border_t_1()
                    .border_color(t.ink_faint)
                    .bg(t.bg)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(t.ink_muted)
                                    .child(format!("{} words", word_count)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(t.ink_muted)
                                    .child(format!("{} chars", char_count)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("btn-footer-add-task")
                                    .icon(IconName::Plus)
                                    .ghost()
                                    .small()
                                    .label("Task")
                                    .tooltip("Add Checklist Item")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.add_checklist_item(cx);
                                    })),
                            ),
                    ),
            )
            // Add Modal / Drawer (Images & Voice Recordings)
            .when(self.show_add_modal, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(hsla(0.0, 0.0, 0.0, 0.35))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(320.0))
                                .p_6()
                                .bg(t.bg)
                                .border_2()
                                .border_color(t.ink)
                                .shadow_xl()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_bold()
                                                .text_color(t.ink)
                                                .child("Attach Media"),
                                        )
                                        .child(
                                            Button::new("btn-close-modal")
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_add_modal = false;
                                                    cx.notify();
                                                })),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("btn-modal-add-image")
                                        .cursor_pointer()
                                        .p_3()
                                        .border_1()
                                        .border_color(t.ink)
                                        .text_center()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .hover(move |s| s.bg(t.bg_hover))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.add_sample_image(cx);
                                        }))
                                        .child("Add image"),
                                )
                                .child(
                                    div()
                                        .id("btn-modal-add-voice")
                                        .cursor_pointer()
                                        .p_3()
                                        .border_1()
                                        .border_color(t.ink)
                                        .text_center()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .hover(move |s| s.bg(t.bg_hover))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.add_sample_voice_recording(cx);
                                        }))
                                        .child("Add voice recording"),
                                ),
                        ),
                )
            })
    }
}

fn this_save(note: &Note, storage: &Storage) -> std::io::Result<()> {
    storage.save(note)
}

impl Render for NotesApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            .relative()
            .child(
                TitleBar::new()
                    .bg(t.bg)
                    .border_color(t.ink_faint)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .child(
                                if let Some(path) = &self.logo_png_path {
                                    div()
                                        .w(px(16.0))
                                        .h(px(16.0))
                                        .rounded_sm()
                                        .overflow_hidden()
                                        .child(
                                            img(path.clone())
                                                .w(px(16.0))
                                                .h(px(16.0)),
                                        )
                                        .into_any_element()
                                } else {
                                    div()
                                        .w(px(14.0))
                                        .h(px(14.0))
                                        .rounded_sm()
                                        .bg(t.ink)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(t.bg)
                                        .child("N")
                                        .into_any_element()
                                },
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .child("Notty"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .child(match self.current_screen {
                        CurrentScreen::Home => self.render_home(cx).into_any_element(),
                        CurrentScreen::Browse => self.render_browse(cx).into_any_element(),
                        CurrentScreen::Editor => self.render_editor(cx).into_any_element(),
                    }),
            )
            .when(self.show_auth_modal, |this| {
                this.child(self.render_auth_modal(&t, cx))
            })
    }
}

#[cfg(target_os = "windows")]
mod win32_fix {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    extern "system" {
        fn ShowWindow(hwnd: isize, n_cmd_show: i32) -> i32;
        fn SetForegroundWindow(hwnd: isize) -> i32;
        fn BringWindowToTop(hwnd: isize) -> i32;
        fn LoadImageW(
            hinst: isize,
            lpszname: *const u16,
            utype: u32,
            cx: i32,
            cy: i32,
            fuload: u32,
        ) -> isize;
        fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
    }

    const IMAGE_ICON: u32 = 1;
    const LR_LOADFROMFILE: u32 = 0x00000010;
    const LR_DEFAULTSIZE: u32 = 0x00000040;
    const WM_SETICON: u32 = 0x0080;
    const ICON_SMALL: usize = 0;
    const ICON_BIG: usize = 1;

    pub fn show_and_focus_window(window: &gpui_kit::Window) {
        if let Ok(handle) = HasWindowHandle::window_handle(window) {
            if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                let hwnd = handle.hwnd.get();
                unsafe {
                    ShowWindow(hwnd, 5 /* SW_SHOW */);
                    BringWindowToTop(hwnd);
                    SetForegroundWindow(hwnd);

                    // Load and apply the app icon to the window and taskbar
                    let icon_path = std::path::Path::new("assests/app_icon.ico");
                    if icon_path.exists() {
                        let path_wide: Vec<u16> = OsStr::new(icon_path)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();

                        let hicon_big = LoadImageW(
                            0,
                            path_wide.as_ptr(),
                            IMAGE_ICON,
                            0,
                            0,
                            LR_LOADFROMFILE | LR_DEFAULTSIZE,
                        );
                        if hicon_big != 0 {
                            SendMessageW(hwnd, WM_SETICON, ICON_BIG, hicon_big);
                        }

                        let hicon_small = LoadImageW(
                            0,
                            path_wide.as_ptr(),
                            IMAGE_ICON,
                            16,
                            16,
                            LR_LOADFROMFILE,
                        );
                        if hicon_small != 0 {
                            SendMessageW(hwnd, WM_SETICON, ICON_SMALL, hicon_small);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    application()
        .with_assets(assets::Assets)
        .run(|cx| {
            init(cx);

            let mut window_options = TitleBar::window_options();
            // Modern desktop rectangular proportion (wide layout for comfortable note taking and split view)
            window_options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1000.0), px(680.0)),
                cx,
            )));
            window_options.focus = true;
            window_options.show = true;

            let (handle, _) = open_window(
                window_options,
                cx,
                |window, cx| {
                    #[cfg(target_os = "windows")]
                    win32_fix::show_and_focus_window(window);

                    window.activate_window();
                    cx.new(|cx| NotesApp::new(window, cx))
                },
            )
            .expect("Failed to open window");

            #[cfg(target_os = "windows")]
            handle
                .update(cx, |_, window, _| {
                    win32_fix::show_and_focus_window(window);
                })
                .ok();

            cx.activate(true);
        });
}
