use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::*;

pub fn render_bottom_dock(
    _this: &NotesApp,
    filtered_count: usize,
    total_count: usize,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
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
                .child(if filtered_count == total_count {
                    format!("Showing all {} notes", total_count)
                } else {
                    format!("Showing {} of {} notes", filtered_count, total_count)
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
        )
}
