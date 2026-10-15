use gpui_kit::base::StyledExt;
use gpui_kit::*;
use std::sync::Arc;
use crate::theme::AppTheme;

pub fn render_logo(t: &AppTheme, logo_png_path: Option<&std::path::PathBuf>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            if let Some(path) = logo_png_path {
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
                        img(path.clone())
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

pub fn render_zen_lamp(t: &AppTheme, lamp_svg_data: Option<&Arc<[u8]>>) -> impl IntoElement {
    if let Some(svg_data) = lamp_svg_data {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w(px(220.0))
            .h(px(280.0))
            .child(
                svg()
                    .data(&**svg_data)
                    .w(px(200.0))
                    .h(px(260.0))
                    .text_color(t.ink),
            )
            .into_any_element()
    } else {
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

#[allow(dead_code)]
pub fn render_vertical_tagline(t: &AppTheme) -> impl IntoElement {
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
        .child(
            div()
                .w(px(1.0))
                .h(px(18.0))
                .bg(t.ink_faint)
                .mb_2(),
        )
        .children(word1.into_iter().map(|ch| {
            div()
                .text_xs()
                .font_bold()
                .text_color(t.ink_muted)
                .line_height(px(15.0))
                .child(ch)
        }))
        .child(
            div()
                .my_2()
                .w(px(3.0))
                .h(px(3.0))
                .rounded_full()
                .bg(t.highlight),
        )
        .children(word2.into_iter().map(|ch| {
            div()
                .text_xs()
                .font_bold()
                .text_color(t.ink_muted)
                .line_height(px(15.0))
                .child(ch)
        }))
        .child(
            div()
                .w(px(1.0))
                .h(px(18.0))
                .bg(t.ink_faint)
                .mt_2(),
        )
}
