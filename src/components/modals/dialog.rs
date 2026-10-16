use crate::theme::AppTheme;
use gpui_kit::*;

pub fn modal_dialog<F>(
    id: &'static str,
    width_px: f32,
    t: &AppTheme,
    on_backdrop_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    content_builder: F,
) -> impl IntoElement
where
    F: FnOnce() -> Div,
{
    div()
        .id(SharedString::from(format!("{}-backdrop", id)))
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.75))
        .flex()
        .items_center()
        .justify_center()
        .on_click(on_backdrop_click)
        .child(
            div()
                .id(SharedString::from(format!("{}-card", id)))
                .w(px(width_px))
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
                .child(content_builder()),
        )
}
