use crate::models::Note;
use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::assets::IconName as FullIconName;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    IconName, Sizable,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl NotesApp {
    /// Renders a single note card for the Grid view.
    pub(crate) fn render_note_card(
        &self,
        note: &Note,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let note_id = note.id.clone();
        let note_id_view = note.id.clone();
        let note_id_edit = note.id.clone();
        let note_id_del = note.id.clone();
        let title = note.display_title().to_string();
        let date = note.formatted_date();
        let is_active = self.selected_id.as_deref() == Some(&note_id);
        let snippet = note.snippet();
        let todo_blocks: Vec<_> = note
            .blocks
            .iter()
            .filter_map(|b| match &b.kind {
                crate::models::BlockKind::Todo { checked, .. } => Some(*checked),
                _ => None,
            })
            .collect();
        let _has_checklist = !todo_blocks.is_empty() || !note.checklist.is_empty();
        let checklist_summary = if !todo_blocks.is_empty() {
            let done = todo_blocks.iter().filter(|&&c| c).count();
            let total = todo_blocks.len();
            Some(format!("{}/{}", done, total))
        } else if !note.checklist.is_empty() {
            let done = note.checklist.iter().filter(|i| i.checked).count();
            let total = note.checklist.len();
            Some(format!("{}/{}", done, total))
        } else {
            None
        };

        div()
            .id(SharedString::from(format!("browse-grid-card-{}", note_id)))
            .flex_1()
            .min_w(px(230.0))
            .max_w(px(360.0))
            .h(px(168.0))
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
                            .text_sm()
                            .font_bold()
                            .text_color(t.ink)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .line_height(relative(1.4))
                            .text_color(t.ink_muted)
                            .h(px(56.0))
                            .overflow_hidden()
                            .child(snippet),
                    ),
            )
            // Card bottom meta & action dock
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pt_3()
                    .border_t_1()
                    .border_color(t.ink_faint)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_xs().text_color(t.ink_muted).child(date))
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
                            .id(SharedString::from(format!(
                                "browse-grid-actions-{}",
                                note_id_del
                            )))
                            .flex()
                            .items_center()
                            .gap_0p5()
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-grid-view-{}",
                                    note_id_view
                                )))
                                .icon(IconName::Eye)
                                .ghost()
                                .small()
                                .tooltip("View Note")
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.open_note(&note_id_view, window, cx);
                                    },
                                )),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-grid-edit-{}",
                                    note_id_edit
                                )))
                                .icon(FullIconName::Pencil)
                                .ghost()
                                .small()
                                .tooltip("Edit Note")
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.open_note(&note_id_edit, window, cx);
                                    },
                                )),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-grid-del-{}",
                                    note_id_del
                                )))
                                .icon(FullIconName::Trash)
                                .ghost()
                                .small()
                                .tooltip("Delete Note")
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.note_to_delete = Some(note_id_del.clone());
                                        cx.notify();
                                    },
                                )),
                            ),
                    ),
            )
    }

    /// Renders a single note row for the List view.
    pub(crate) fn render_note_list_item(
        &self,
        note: &Note,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let note_id = note.id.clone();
        let note_id_view = note.id.clone();
        let note_id_edit = note.id.clone();
        let note_id_del = note.id.clone();
        let title = note.display_title().to_string();
        let date = note.formatted_date();
        let is_active = self.selected_id.as_deref() == Some(&note_id);
        let todo_blocks: Vec<_> = note
            .blocks
            .iter()
            .filter_map(|b| match &b.kind {
                crate::models::BlockKind::Todo { checked, .. } => Some(*checked),
                _ => None,
            })
            .collect();
        let has_checklist = !todo_blocks.is_empty() || !note.checklist.is_empty();

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
                    .pr_8()
                    .child(
                        div()
                            .text_sm()
                            .font_bold()
                            .text_color(t.ink)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .line_height(relative(1.35))
                            .text_color(t.ink_muted)
                            .max_h(px(32.0))
                            .overflow_hidden()
                            .child(note.snippet_max(180)),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_4()
                    .when(has_checklist, |this| {
                        let total = if !todo_blocks.is_empty() {
                            todo_blocks.len()
                        } else {
                            note.checklist.len()
                        };
                        let done = if !todo_blocks.is_empty() {
                            todo_blocks.iter().filter(|&&c| c).count()
                        } else {
                            note.checklist.iter().filter(|i| i.checked).count()
                        };
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
                            .w(px(85.0))
                            .text_xs()
                            .text_color(t.ink_muted)
                            .child(date),
                    )
                    // Note Actions: Eye (View), Pencil (Edit), Trash (Delete)
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "browse-actions-{}",
                                note_id_del
                            )))
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-browse-view-{}",
                                    note_id_view
                                )))
                                .icon(IconName::Eye)
                                .ghost()
                                .small()
                                .tooltip("View Note")
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.open_note(&note_id_view, window, cx);
                                    },
                                )),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-browse-edit-{}",
                                    note_id_edit
                                )))
                                .icon(FullIconName::Pencil)
                                .ghost()
                                .small()
                                .tooltip("Edit Note")
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.open_note(&note_id_edit, window, cx);
                                    },
                                )),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "btn-browse-del-{}",
                                    note_id_del
                                )))
                                .icon(FullIconName::Trash)
                                .ghost()
                                .small()
                                .tooltip("Delete Note")
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.note_to_delete = Some(note_id_del.clone());
                                        cx.notify();
                                    },
                                )),
                            ),
                    ),
            )
    }
}
