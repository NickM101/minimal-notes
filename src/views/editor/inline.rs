use crate::markdown::{parse_inline_tokens, InlineTok};
use crate::theme::{AppTheme, LIGHT_INK};
use gpui_kit::base::StyledExt;
use gpui_kit::*;

pub fn render_inline_spans(text: &str, t: &AppTheme) -> impl IntoElement {
    let tokens = parse_inline_tokens(text);
    div()
        .flex()
        .flex_wrap()
        .items_baseline()
        .gap_x_1()
        .gap_y_0p5()
        .children(tokens.into_iter().map(|tok| {
            match tok {
                InlineTok::Plain(s) => div()
                    .text_color(t.ink)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Bold(s) => div()
                    .font_bold()
                    .text_color(t.ink)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Italic(s) => div()
                    .italic()
                    .text_color(t.ink)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Highlight(s) => div()
                    .bg(t.highlight)
                    .px_1()
                    .py_0p5()
                    .rounded_sm()
                    .text_xs()
                    .font_bold()
                    .text_color(LIGHT_INK)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Strike(s) => div()
                    .line_through()
                    .text_color(t.ink_muted)
                    .child(s.to_string())
                    .into_any_element(),
                InlineTok::Code(s) => div()
                    .font_family(SharedString::from("Consolas, Courier New, monospace"))
                    .text_xs()
                    .bg(t.surface)
                    .border_1()
                    .border_color(t.ink_faint)
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .text_color(t.ink)
                    .child(s.to_string())
                    .into_any_element(),
            }
        }))
}
