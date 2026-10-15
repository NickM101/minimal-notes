use gpui_kit::assets::IconName as FullIconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, input::Input, Icon, IconName, Sizable};
use crate::models::Note;
use crate::theme::CurrentScreen;
use crate::{BrowseLayout, NotesApp};

impl NotesApp {
    pub(crate) fn render_browse(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
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
        let filtered_count = filtered_notes.len();

        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(t.bg)
            .text_color(t.ink)
            // Navigation bar
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_3p5()
                    .border_b_1()
                    .border_color(t.ink_faint)
                    .child(
                        div()
                            .id("btn-browse-back")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1p5()
                            .rounded_md()
                            .text_color(t.ink_muted)
                            .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                            .active(move |s| s.bg(t.bg_active))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_screen = CurrentScreen::Home;
                                cx.notify();
                            }))
                            .child(Icon::new(IconName::ChevronLeft).size_4())
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child("Back to Home"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            // Search toggle button (dropdown search bar)
                            .child(
                                Button::new("btn-browse-search-toggle")
                                    .icon(IconName::Search)
                                    .ghost()
                                    .tooltip(if self.browse_search_open { "Hide search" } else { "Search notes" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.browse_search_open = !this.browse_search_open;
                                        cx.notify();
                                    })),
                            )
                            // Layout toggle: Grid vs List
                            .child(
                                Button::new("btn-browse-layout-toggle")
                                    .icon(match self.browse_layout {
                                        BrowseLayout::List => FullIconName::LayoutGrid,
                                        BrowseLayout::Grid => FullIconName::LayoutList,
                                    })
                                    .ghost()
                                    .tooltip(match self.browse_layout {
                                        BrowseLayout::List => "Switch to Grid view",
                                        BrowseLayout::Grid => "Switch to List view",
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.browse_layout = match this.browse_layout {
                                            BrowseLayout::List => BrowseLayout::Grid,
                                            BrowseLayout::Grid => BrowseLayout::List,
                                        };
                                        cx.notify();
                                    })),
                            )
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
                            .child(self.render_avatar_button(&t, cx)),
                    ),
            )
            // Dropdown Search Bar: Shown only when user clicks the Search button
            .when(self.browse_search_open, |this| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .px_6()
                        .py_2p5()
                        .border_b_1()
                        .border_color(t.ink_faint)
                        .bg(t.surface)
                        .child(
                            // Search box with rounded border and subtle background
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .border_1()
                                .border_color(t.ink_faint)
                                .bg(t.bg)
                                .child(Icon::new(IconName::Search).size_4().text_color(t.ink_muted))
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
                        .child(
                            Button::new("btn-close-search-bar")
                                .icon(IconName::Close)
                                .ghost()
                                .small()
                                .tooltip("Close search")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.browse_search_open = false;
                                    this.search_query.clear();
                                    this.search_state.update(cx, |s, cx| {
                                        s.set_value("", window, cx);
                                    });
                                    cx.notify();
                                })),
                        ),
                )
            })
            // Scrollable Notes Container (Grid or List)
            .child(
                div()
                    .id("browse-notes-scroll-area")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .when(filtered_notes.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_20()
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
                    .when(!filtered_notes.is_empty(), |this| {
                        match self.browse_layout {
                            // ─── LIST VIEW ───
                            BrowseLayout::List => {
                                this.child(
                                    div()
                                        .id("browse-notes-list")
                                        .flex()
                                        .flex_col()
                                        .pb_4()
                                        .children(filtered_notes.iter().map(|note| {
                                            self.render_note_list_item(note, &t, cx)
                                        })),
                                )
                            }
                            // ─── GRID VIEW (CARDS) ───
                            BrowseLayout::Grid => {
                                this.child(
                                    div()
                                        .id("browse-notes-grid")
                                        .flex()
                                        .flex_wrap()
                                        .gap_4()
                                        .p_6()
                                        .pb_16()
                                        .children(filtered_notes.iter().map(|note| {
                                            self.render_note_card(note, &t, cx)
                                        })),
                                )
                            }
                        }
                    }),
            )
            // Bottom Action Dock: Always visible and comfortably positioned with stats
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_3()
                    .border_t_1()
                    .border_color(t.ink_faint)
                    .bg(t.bg)
                    .child(
                        div()
                            .text_xs()
                            .text_color(t.ink_muted)
                            .child(if filtered_count == self.notes.len() {
                                format!("Showing all {} notes", self.notes.len())
                            } else {
                                format!("Showing {} of {} notes", filtered_count, self.notes.len())
                            }),
                    )
                    .child(
                        div()
                            .id("btn-browse-new-bottom")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_6()
                            .py_2()
                            .rounded_md()
                            .border_2()
                            .border_color(t.ink)
                            .bg(t.bg)
                            .hover(move |s| s.bg(t.bg_hover))
                            .active(move |s| s.bg(t.bg_active))
                            .text_xs()
                            .font_bold()
                            .text_color(t.ink)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_new_note(window, cx);
                            }))
                            .child(Icon::new(IconName::Plus).size_3p5())
                            .child("New Note"),
                    ),
            )
    }


}

