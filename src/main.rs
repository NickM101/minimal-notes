mod app;
mod components;
mod markdown;
mod models;
mod platform;
mod storage;
mod theme;
mod views;

use std::sync::Arc;
use std::time::Instant;

use gpui_kit::component::{
    input::{EditorState, InputEvent, InputState},
    theme::{Theme, ThemeMode}, TitleBar,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use models::Note;
use storage::Storage;
use theme::{AppTheme, CurrentScreen, EditorModeView};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BrowseLayout {
    List,
    Grid,
}

pub struct NotesApp {
    storage: Storage,
    notes: Vec<Note>,
    selected_id: Option<String>,
    search_query: String,
    current_screen: CurrentScreen,
    browse_layout: BrowseLayout,
    browse_search_open: bool,
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

    // Settings Modal State
    show_settings_modal: bool,
    auto_save_interval_sec: u32,
    font_size_idx: usize,

    // Update Checker Modal State
    show_update_modal: bool,
    is_checking_update: bool,
    update_checked: bool,

    // Note Deletion Confirmation Modal State
    note_to_delete: Option<String>,

    _subscriptions: Vec<Subscription>,
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
            browse_layout: BrowseLayout::List,
            browse_search_open: false,
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
            show_settings_modal: false,
            auto_save_interval_sec: 2,
            font_size_idx: 1, // Medium (default)
            show_update_modal: false,
            is_checking_update: false,
            update_checked: false,
            note_to_delete: None,
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


    fn selected_note(&self) -> Option<&Note> {
        let id = self.selected_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    fn render_logo(&self, t: &AppTheme) -> impl IntoElement {
        components::branding::render_logo(t, self.logo_png_path.as_ref())
    }

    fn render_zen_lamp(&self, t: &AppTheme) -> impl IntoElement {
        components::branding::render_zen_lamp(t, self.lamp_svg_data.as_ref())
    }

    #[allow(dead_code)]
    fn render_vertical_tagline(&self, t: &AppTheme) -> impl IntoElement {
        components::branding::render_vertical_tagline(t)
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
            .child(self.render_titlebar(&t, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .size_full()
                    .overflow_hidden()
                    .child(match self.current_screen {
                        CurrentScreen::Home => self.render_home(cx).into_any_element(),
                        CurrentScreen::Browse => self.render_browse(cx).into_any_element(),
                        CurrentScreen::Editor => self.render_editor(cx).into_any_element(),
                    }),
            )
            .when(self.show_auth_modal, |this| {
                this.child(self.render_auth_modal(&t, cx))
            })
            .when(self.show_settings_modal, |this| {
                this.child(self.render_settings_modal(&t, cx))
            })
            .when(self.show_update_modal, |this| {
                this.child(self.render_update_modal(&t, cx))
            })
            .when(self.note_to_delete.is_some(), |this| {
                this.child(self.render_delete_modal(&t, cx))
            })
    }
}

fn main() {
    application()
        .with_assets(assets::AllAssets)
        .run(|cx| {
            init(cx);

            let mut window_options = TitleBar::window_options();
            // Modern desktop rectangular proportion (width calibrated to fit 4 cards across in browse grid)
            window_options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1100.0), px(700.0)),
                cx,
            )));
            window_options.focus = true;
            window_options.show = true;

            let (handle, _) = open_window(
                window_options,
                cx,
                |window, cx| {
                    #[cfg(target_os = "windows")]
                    platform::win32::show_and_focus_window(window);

                    window.activate_window();
                    cx.new(|cx| NotesApp::new(window, cx))
                },
            )
            .expect("Failed to open window");

            #[cfg(target_os = "windows")]
            handle
                .update(cx, |_, window, _| {
                    platform::win32::show_and_focus_window(window);
                })
                .ok();

            cx.activate(true);
        });
}
