pub mod dock;
pub mod nav;
pub mod search_bar;

use gpui_kit::*;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder;
use crate::models::Note;
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
                        || n.plain_text().to_lowercase().contains(&self.search_query)
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
            .child(nav::render_browse_nav(self, &t, cx))
            // Dropdown Search Bar: Shown only when user clicks the Search button
            .when(self.browse_search_open, |this| {
                this.child(search_bar::render_search_bar(self, &t, cx))
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
            // Bottom Action Dock
            .child(dock::render_bottom_dock(self, filtered_count, self.notes.len(), &t, cx))
    }
}
