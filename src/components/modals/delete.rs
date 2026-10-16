use crate::components::modals::dialog::modal_dialog;
use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::*;

impl NotesApp {
    pub(crate) fn render_delete_modal(&self, t: &AppTheme, cx: &mut Context<Self>) -> AnyElement {
        let note_id = self.note_to_delete.clone().unwrap_or_default();
        let target_title = self
            .notes
            .iter()
            .find(|n| n.id == note_id)
            .map(|n| n.display_title().to_string())
            .unwrap_or_else(|| "Untitled Note".to_string());
        let note_id_del = note_id.clone();

        modal_dialog(
            "delete-modal",
            380.0,
            t,
            cx.listener(|this, _, _, cx| {
                this.note_to_delete = None;
                cx.notify();
            }),
            move || {
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Delete).size_5().text_color(t.ink))
                            .child(
                                div()
                                    .text_base()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .child("Delete Note?"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .line_height(relative(1.5))
                            .text_color(t.ink_muted)
                            .child(format!(
                                "Are you sure you want to permanently delete \"{}\"? This action cannot be undone.",
                                target_title
                            )),
                    )
                    // Action Buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_3()
                            .pt_2()
                            .child(
                                div()
                                    .id("btn-cancel-delete-modal")
                                    .cursor_pointer()
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .text_color(t.ink)
                                    .text_xs()
                                    .font_medium()
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.note_to_delete = None;
                                        cx.notify();
                                    }))
                                    .child("Cancel"),
                            )
                            .child(
                                div()
                                    .id("btn-confirm-delete-modal")
                                    .cursor_pointer()
                                    .px_4()
                                    .py_2()
                                    .rounded_md()
                                    .bg(hsla(0.0, 0.75, 0.45, 1.0))
                                    .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                    .text_xs()
                                    .font_bold()
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.delete_note_by_id(&note_id_del, window, cx);
                                        this.note_to_delete = None;
                                        cx.notify();
                                    }))
                                    .child("Delete Note"),
                            ),
                    )
            },
        )
        .into_any_element()
    }
}
