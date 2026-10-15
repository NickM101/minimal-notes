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
                                        .children(filtered_notes.into_iter().map(|note| {
                                            let note_id = note.id.clone();
                                            let note_id_view = note.id.clone();
                                            let note_id_edit = note.id.clone();
                                            let note_id_del = note.id.clone();
                                            let title = note.display_title().to_string();
                                            let date = note.formatted_date();
                                            let is_active = self.selected_id.as_deref() == Some(&note_id);

                                            div()
                                                .id(SharedString::from(format!("browse-item-{}", note_id)))
                                                .group("browse-row")
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .px_6()
                                                .py_3p5()
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
                                                        .flex_1()
                                                        .min_w_0()
                                                        .flex_col()
                                                        .gap_1()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_bold()
                                                                        .text_color(t.ink)
                                                                        .overflow_hidden()
                                                                        .text_ellipsis()
                                                                        .child(title),
                                                                )
                                                                .when(is_active, |this| {
                                                                    this.child(
                                                                        div()
                                                                            .text_xs()
                                                                            .px_1p5()
                                                                            .py_0p5()
                                                                            .rounded_sm()
                                                                            .bg(t.ink)
                                                                            .text_color(t.bg)
                                                                            .font_medium()
                                                                            .child("Active"),
                                                                    )
                                                                }),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(t.ink_muted)
                                                                .overflow_hidden()
                                                                .text_ellipsis()
                                                                .child(note.snippet()),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_3()
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
                                                        )
                                                        // Note Actions: Eye (View), Pencil (Edit), Trash (Delete)
                                                        .child(
                                                            div()
                                                                .id(SharedString::from(format!("browse-actions-{}", note_id_del)))
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-browse-view-{}", note_id_view)))
                                                                        .icon(IconName::Eye)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("View Note")
                                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                                            this.open_note(&note_id_view, window, cx);
                                                                        })),
                                                                )
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-browse-edit-{}", note_id_edit)))
                                                                        .icon(FullIconName::Pencil)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("Edit Note")
                                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                                            this.open_note(&note_id_edit, window, cx);
                                                                        })),
                                                                )
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-browse-del-{}", note_id_del)))
                                                                        .icon(FullIconName::Trash)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("Delete Note")
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            this.note_to_delete = Some(note_id_del.clone());
                                                                            cx.notify();
                                                                        })),
                                                                ),
                                                        ),
                                                )
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
                                        .pb_8()
                                        .children(filtered_notes.into_iter().map(|note| {
                                            let note_id = note.id.clone();
                                            let note_id_view = note.id.clone();
                                            let note_id_edit = note.id.clone();
                                            let note_id_del = note.id.clone();
                                            let title = note.display_title().to_string();
                                            let date = note.formatted_date();
                                            let is_active = self.selected_id.as_deref() == Some(&note_id);
                                            let snippet = note.snippet();
                                            let has_checklist = !note.checklist.is_empty();
                                            let checklist_summary = if has_checklist {
                                                let done = note.checklist.iter().filter(|i| i.checked).count();
                                                let total = note.checklist.len();
                                                Some(format!("{}/{}", done, total))
                                            } else {
                                                None
                                            };

                                            div()
                                                .id(SharedString::from(format!("browse-grid-card-{}", note_id)))
                                                .w(px(240.0))
                                                .h(px(160.0))
                                                .flex()
                                                .flex_col()
                                                .justify_between()
                                                .p_4()
                                                .rounded_lg()
                                                .border_1()
                                                .border_color(if is_active { t.ink } else { t.ink_faint })
                                                .bg(if is_active { t.surface } else { t.bg })
                                                .cursor_pointer()
                                                .hover(move |s| s.bg(t.bg_hover).border_color(t.ink_muted))
                                                .active(move |s| s.bg(t.bg_active))
                                                .on_click(cx.listener(move |this, _, window, cx| {
                                                    this.open_note(&note_id, window, cx);
                                                }))
                                                // Card top content
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_bold()
                                                                        .text_color(t.ink)
                                                                        .overflow_hidden()
                                                                        .text_ellipsis()
                                                                        .child(title),
                                                                )
                                                                .when(is_active, |this| {
                                                                    this.child(
                                                                        div()
                                                                            .text_xs()
                                                                            .px_1p5()
                                                                            .py_0p5()
                                                                            .rounded_sm()
                                                                            .bg(t.ink)
                                                                            .text_color(t.bg)
                                                                            .font_medium()
                                                                            .child("Active"),
                                                                    )
                                                                }),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(t.ink_muted)
                                                                .h(px(40.0))
                                                                .overflow_hidden()
                                                                .text_ellipsis()
                                                                .child(snippet),
                                                        ),
                                                )
                                                // Card bottom meta & action dock
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .pt_2()
                                                        .border_t_1()
                                                        .border_color(t.ink_faint)
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_2()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(t.ink_muted)
                                                                        .child(date),
                                                                )
                                                                .when_some(checklist_summary, |this, sum| {
                                                                    this.child(
                                                                        div()
                                                                            .text_xs()
                                                                            .px_1p5()
                                                                            .py_0p5()
                                                                            .rounded_sm()
                                                                            .border_1()
                                                                            .border_color(t.ink_faint)
                                                                            .text_color(t.ink_muted)
                                                                            .child(sum),
                                                                    )
                                                                }),
                                                        )
                                                        .child(
                                                            div()
                                                                .id(SharedString::from(format!("browse-grid-actions-{}", note_id_del)))
                                                                .flex()
                                                                .items_center()
                                                                .gap_0p5()
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-grid-view-{}", note_id_view)))
                                                                        .icon(IconName::Eye)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("View Note")
                                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                                            this.open_note(&note_id_view, window, cx);
                                                                        })),
                                                                )
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-grid-edit-{}", note_id_edit)))
                                                                        .icon(FullIconName::Pencil)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("Edit Note")
                                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                                            this.open_note(&note_id_edit, window, cx);
                                                                        })),
                                                                )
                                                                .child(
                                                                    Button::new(SharedString::from(format!("btn-grid-del-{}", note_id_del)))
                                                                        .icon(FullIconName::Trash)
                                                                        .ghost()
                                                                        .small()
                                                                        .tooltip("Delete Note")
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            this.note_to_delete = Some(note_id_del.clone());
                                                                            cx.notify();
                                                                        })),
                                                                ),
                                                        ),
                                                )
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

