use crate::markdown::{parse_markdown_blocks, MarkdownBlock};
use crate::theme::{AppTheme, LIGHT_INK};
use crate::views::editor::inline::render_inline_spans;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Icon, IconName, Sizable,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::time::Instant;

impl NotesApp {
    #[allow(dead_code)]
    pub(crate) fn render_notion_blocks(
        &self,
        content: &str,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let blocks = parse_markdown_blocks(content);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .py_2()
            .children(blocks.into_iter().enumerate().map(|(idx, block)| {
                match block {
                    MarkdownBlock::Blank => div().h(px(8.0)).into_any_element(),
                    MarkdownBlock::Heading(level, text) => {
                        let text_clone = text.clone();
                        match level {
                            1 => div()
                                .mt_4()
                                .mb_2()
                                .text_2xl()
                                .font_bold()
                                .text_color(t.ink)
                                .border_b_1()
                                .border_color(t.ink_faint)
                                .pb_1()
                                .child(render_inline_spans(&text_clone, t))
                                .into_any_element(),
                            2 => div()
                                .mt_3()
                                .mb_1p5()
                                .text_xl()
                                .font_bold()
                                .text_color(t.ink)
                                .child(render_inline_spans(&text_clone, t))
                                .into_any_element(),
                            _ => div()
                                .mt_2()
                                .mb_1()
                                .text_base()
                                .font_bold()
                                .text_color(t.ink)
                                .child(render_inline_spans(&text_clone, t))
                                .into_any_element(),
                        }
                    }
                    MarkdownBlock::Checklist {
                        line_idx,
                        is_checked,
                        text,
                    } => {
                        let is_checked_val = is_checked;
                        let text_copy = text.clone();
                        div()
                            .id(SharedString::from(format!("notion-chk-{}", line_idx)))
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_1()
                            .child(
                                div()
                                    .id(SharedString::from(format!("notion-chk-box-{}", line_idx)))
                                    .cursor_pointer()
                                    .mt_0p5()
                                    .w(px(18.0))
                                    .h(px(18.0))
                                    .border_1()
                                    .border_color(if is_checked_val { t.ink } else { t.ink_muted })
                                    .bg(if is_checked_val { t.ink } else { t.bg })
                                    .rounded_sm()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .hover(move |s| s.border_color(t.ink).opacity(0.85))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.toggle_markdown_checkbox(line_idx, window, cx);
                                    }))
                                    .when(is_checked_val, |this| {
                                        this.child(
                                            Icon::new(IconName::Check).size_3().text_color(t.bg),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(if is_checked_val { t.ink_muted } else { t.ink })
                                    .when(is_checked_val, |s| s.line_through())
                                    .child(render_inline_spans(&text_copy, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Bullet(text) => {
                        let text_clone = text.clone();
                        div()
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_0p5()
                            .child(div().w(px(6.0)).h(px(6.0)).mt_2().rounded_full().bg(t.ink))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Numbered(num, text) => {
                        let text_clone = text.clone();
                        div()
                            .flex()
                            .items_start()
                            .gap_2p5()
                            .my_0p5()
                            .child(
                                div()
                                    .w(px(20.0))
                                    .text_sm()
                                    .font_bold()
                                    .text_color(t.ink_muted)
                                    .child(num),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Quote(text) => {
                        let text_clone = text.clone();
                        div()
                            .my_2()
                            .pl_4()
                            .py_1p5()
                            .border_l_4()
                            .border_color(t.ink)
                            .bg(t.surface)
                            .italic()
                            .text_sm()
                            .line_height(relative(1.6))
                            .text_color(t.ink_muted)
                            .child(render_inline_spans(&text_clone, t))
                            .into_any_element()
                    }
                    MarkdownBlock::Callout { kind: _, text } => {
                        let text_clone = text.clone();
                        div()
                            .my_2p5()
                            .p_3p5()
                            .rounded_md()
                            .border_1()
                            .border_color(t.highlight)
                            .bg(if t.is_dark {
                                hsla(45.0 / 360.0, 0.40, 0.14, 1.0)
                            } else {
                                hsla(52.0 / 360.0, 0.45, 0.88, 1.0)
                            })
                            .flex()
                            .items_start()
                            .gap_3()
                            .child(
                                div()
                                    .mt_0p5()
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded_sm()
                                    .bg(t.highlight)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        Icon::new(IconName::Info).size_3().text_color(LIGHT_INK),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .child(render_inline_spans(&text_clone, t)),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::CodeBlock { language, code } => {
                        let code_clone = code.clone();
                        let lang_upper = if language.is_empty() {
                            "CODE".to_string()
                        } else {
                            language.to_uppercase()
                        };

                        div()
                            .my_3()
                            .rounded_md()
                            .border_1()
                            .border_color(t.ink_faint)
                            .bg(if t.is_dark {
                                hsla(0.0, 0.0, 0.06, 1.0)
                            } else {
                                hsla(54.0 / 360.0, 0.18, 0.74, 1.0)
                            })
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_1p5()
                                    .border_b_1()
                                    .border_color(t.ink_faint)
                                    .bg(if t.is_dark {
                                        hsla(0.0, 0.0, 0.09, 1.0)
                                    } else {
                                        hsla(54.0 / 360.0, 0.20, 0.78, 1.0)
                                    })
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .w(px(6.0))
                                                    .h(px(6.0))
                                                    .rounded_full()
                                                    .bg(t.highlight),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(t.ink_muted)
                                                    .child(lang_upper),
                                            ),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "btn-copy-code-{}",
                                            idx
                                        )))
                                        .icon(IconName::Copy)
                                        .ghost()
                                        .small()
                                        .label("Copy")
                                        .tooltip("Copy snippet to clipboard")
                                        .on_click(
                                            cx.listener(move |this, _, _, cx| {
                                                cx.write_to_clipboard(ClipboardItem::new_string(
                                                    code_clone.clone(),
                                                ));
                                                this.status_message = Some((
                                                    "Code copied!".to_string(),
                                                    Instant::now(),
                                                ));
                                                cx.notify();
                                            }),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .p_3()
                                    .overflow_hidden()
                                    .font_family(SharedString::from(
                                        "Consolas, Courier New, monospace",
                                    ))
                                    .text_xs()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink)
                                    .children(code.lines().enumerate().map(
                                        |(line_num, line_str)| {
                                            div()
                                                .flex()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .w(px(24.0))
                                                        .text_right()
                                                        .text_color(t.ink_faint)
                                                        .child((line_num + 1).to_string()),
                                                )
                                                .child(div().flex_1().child(line_str.to_string()))
                                        },
                                    )),
                            )
                            .into_any_element()
                    }
                    MarkdownBlock::Divider => div()
                        .my_4()
                        .h_px()
                        .w_full()
                        .bg(t.ink_faint)
                        .into_any_element(),
                    MarkdownBlock::Paragraph(text) => {
                        let text_clone = text.clone();
                        div()
                            .my_1()
                            .text_sm()
                            .line_height(relative(1.6))
                            .text_color(t.ink)
                            .child(render_inline_spans(&text_clone, t))
                            .into_any_element()
                    }
                }
            }))
    }
}
