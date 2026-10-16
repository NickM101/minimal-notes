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
    input::{InputEvent, InputState},
    theme::{Theme, ThemeMode},
    TitleBar,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use models::Note;
use storage::Storage;
use theme::{AppTheme, CurrentScreen};

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
    zen_focus_mode: bool,

    // Status message for save feedback
    status_message: Option<(String, Instant)>,
    _save_task: Option<Task<()>>,

    search_state: Entity<InputState>,
    title_state: Entity<InputState>,
    block_input_states: Vec<app::blocks::BlockInputHandle>,
    slash_menu_target: Option<(String, String)>, // (block_id, filter_query)
    slash_menu_selection: Option<usize>,         // keyboard-selected item index in slash menu
    hovered_block_id: Option<String>,
    focused_block_id: Option<String>, // which block currently has keyboard focus

    // Embedded assets (compile-time): the binary is self-contained and
    // never depends on the working directory.
    lamp_svg_data: Arc<[u8]>,
    logo_png: Option<Arc<Image>>,

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

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search notes..."));

        let title_state = cx.new(|cx| InputState::new(window, cx).placeholder("Title..."));

        title_state.update(cx, |s, cx| {
            s.set_value(&initial_title, window, cx);
        });

        let mut subscriptions = Vec::new();

        subscriptions.push(
            cx.subscribe_in(&search_state, window, |this, state, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    this.search_query = state.read(cx).value().to_string().to_lowercase();
                    cx.notify();
                }
            }),
        );

        subscriptions.push(
            cx.subscribe_in(&title_state, window, |this, state, event, _, cx| {
                if this.is_syncing {
                    return;
                }
                if matches!(event, InputEvent::Change) {
                    let new_title = state.read(cx).value().to_string();
                    if let Some(id) = &this.selected_id {
                        if let Some(note) = this.notes.iter_mut().find(|n| n.id == *id) {
                            note.title = new_title;
                            note.updated_at = chrono::Local::now();
                            this.schedule_save(cx);
                            cx.notify();
                        }
                    }
                }
            }),
        );

        // Assets are embedded at compile time — nothing is read from disk.
        let lamp_svg_data: Arc<[u8]> =
            Arc::from(include_bytes!("../assets/zen_lamp.svg").as_slice());
        let logo_png = Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../assets/zen_note_logo.png").to_vec(),
        );

        let mut app = Self {
            storage,
            notes,
            selected_id,
            search_query: String::new(),
            current_screen: CurrentScreen::Home,
            browse_layout: BrowseLayout::Grid,
            browse_search_open: false,
            is_syncing: false,
            show_add_modal: false,
            is_dark_mode: false,
            zen_focus_mode: false,
            status_message: None,
            _save_task: None,
            search_state,
            title_state,
            block_input_states: Vec::new(),
            slash_menu_target: None,
            slash_menu_selection: None,
            hovered_block_id: None,
            focused_block_id: None,
            lamp_svg_data,
            logo_png: Some(Arc::new(logo_png)),
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
        };

        app.reconcile_block_inputs(window, cx);
        app
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
        components::branding::render_logo(t, self.logo_png.as_ref())
    }

    fn render_zen_lamp(&self, t: &AppTheme) -> impl IntoElement {
        components::branding::render_zen_lamp(t, &self.lamp_svg_data)
    }
}

pub(crate) fn this_save(note: &Note, storage: &Storage) -> std::io::Result<()> {
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
    application().with_assets(assets::AllAssets).run(|cx| {
        init(cx);

        let mut window_options = TitleBar::window_options();
        // Modern desktop rectangular proportion (width calibrated to fit 4 cards across in browse grid)
        window_options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1100.0), px(700.0)),
            cx,
        )));
        window_options.window_min_size = Some(size(px(720.0), px(500.0)));
        window_options.focus = true;
        window_options.show = true;

        let (_handle, _) = open_window(window_options, cx, |window, cx| {
            #[cfg(target_os = "windows")]
            platform::win32::show_and_focus_window(window);

            window.activate_window();
            cx.new(|cx| NotesApp::new(window, cx))
        })
        .expect("Failed to open window");

        #[cfg(target_os = "windows")]
        _handle
            .update(cx, |_, window, _| {
                platform::win32::show_and_focus_window(window);
            })
            .ok();

        cx.activate(true);
    });
}
