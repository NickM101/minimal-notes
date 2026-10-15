use std::time::Instant;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{button::{Button, ButtonVariants}, input::Input, Icon, IconName, Sizable};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use crate::models::{Block, BlockKind};
use crate::theme::AppTheme;
use crate::views::editor::inline::render_inline_spans;
use crate::NotesApp;

impl NotesApp {
    /// Renders the note's structured `Vec<Block>` in a centered, clean reading canvas.
    /// Meets the UI/UX spec:
    /// - Max width ~720px, centered
    /// - Paragraph: 16px / 1.6 line height
    /// - Headings: H1 (30px), H2 (24px), H3 (20px), semibold
    /// - Bullet / Numbered: hanging indent, 24px per level
    /// - Todo: Checkbox on left; checked is muted + strikethrough
    /// - Quote: 3px left bar, italic, padded
    /// - Code: Monospace 14px, rounded, code background, language label top right, horizontal scroll
    /// - Divider: 1px line
    /// - Unknown: Rendered gracefully without crashing
    pub(crate) fn render_block_canvas(
        &self,
        blocks: &[Block],
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let slash_target_id = self.slash_menu_target.as_ref().map(|(id, _)| id.clone());
        let slash_query = self.slash_menu_target.as_ref().map(|(_, q)| q.to_lowercase()).unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .py_2()
            .w_full()
            .relative()
            .children(blocks.iter().enumerate().map(|(idx, block)| {
                let block_id = block.id.clone();
                let is_hovered = self.hovered_block_id.as_deref() == Some(&block_id);
                let is_this_slash_target = slash_target_id.as_deref() == Some(&block_id);
                let bid_hover = block_id.clone();
                let bid_action = block_id.clone();

                div()
                    .id(SharedString::from(format!("block-row-{}", block_id)))
                    .flex()
                    .items_start()
                    .relative()
                    .w_full()
                    .group("block-row")
                    .on_hover(cx.listener(move |this, is_over, _window, cx| {
                        if *is_over {
                            if this.hovered_block_id.as_deref() != Some(&bid_hover) {
                                this.hovered_block_id = Some(bid_hover.clone());
                                cx.notify();
                            }
                        } else {
                            if this.hovered_block_id.as_deref() == Some(&bid_hover) {
                                this.hovered_block_id = None;
                                cx.notify();
                            }
                        }
                    }))
                    // Left Gutter: 24px icon handle
                    .child(
                        div()
                            .w(px(24.0))
                            .min_w(px(24.0))
                            .h(px(26.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .mr_1p5()
                            .opacity(if is_hovered { 1.0 } else { 0.0 })
                            .child(
                                div()
                                    .id(SharedString::from(format!("gutter-btn-{}", block_id)))
                                    .cursor_pointer()
                                    .p_0p5()
                                    .rounded_sm()
                                    .hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.15)))
                                    .child(
                                        Icon::new(IconName::Menu)
                                            .size_3p5()
                                            .text_color(t.ink_muted),
                                    )
                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                        // Gutter click opens the slash menu on this block!
                                        this.slash_menu_target = Some((bid_action.clone(), String::new()));
                                        cx.notify();
                                    })),
                            ),
                    )
                    // Block content
                    .child(
                        div()
                            .flex_1()
                            .child(self.render_read_only_block(idx, block, t, cx))
                            // Inline Slash Command Popover Menu directly under this block!
                            .when(is_this_slash_target, |this| {
                                this.child(crate::views::editor::slash_menu::render_slash_menu(&block_id, &slash_query, t, cx))
                            }),
                    )
                    .into_any_element()
            }))
    }

    fn render_read_only_block(
        &self,
        idx: usize,
        block: &Block,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let block_id = block.id.clone();

        let maybe_input_state = self
            .block_input_states
            .iter()
            .find(|(id, _)| id == &block_id)
            .map(|(_, state)| state.clone());

        match &block.kind {
            BlockKind::Paragraph { text } => {
                let text_clone = text.clone();
                let mut block_el = div()
                    .id(SharedString::from(format!("block-p-{}", block_id)))
                    .my_1()
                    .min_h(px(26.0))
                    .text_size(px(16.0))
                    .line_height(relative(1.6))
                    .text_color(t.ink);

                if let Some(state) = maybe_input_state {
                    block_el = block_el.child(
                        Input::new(&state)
                            .bordered(false)
                            .w_full(),
                    );
                } else {
                    if text_clone.trim().is_empty() {
                        block_el = block_el.child(
                            div()
                                .text_color(t.ink_faint)
                                .italic()
                                .child("Type '/' for commands..."),
                        );
                    } else {
                        block_el = block_el.child(render_inline_spans(&text_clone, t));
                    }
                }
                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                block_el = block_el.on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "backspace" {
                        this.handle_block_backspace(&bid_bs, window, cx);
                    } else if event.keystroke.key == "enter" {
                        this.handle_block_enter(&bid_enter, window, cx);
                    } else if event.keystroke.key == "up" {
                        this.handle_block_arrow(&bid_arr, true, window, cx);
                    } else if event.keystroke.key == "down" {
                        this.handle_block_arrow(&bid_arr, false, window, cx);
                    }
                }));
                block_el.into_any_element()
            }
            BlockKind::Heading { level, text } => {
                let text_clone = text.clone();
                let heading_el = match level {
                    1 => div()
                        .mt_5()
                        .mb_2()
                        .text_size(px(30.0))
                        .font_bold()
                        .line_height(relative(1.3))
                        .text_color(t.ink)
                        .border_b_1()
                        .border_color(t.ink_faint)
                        .pb_1p5(),
                    2 => div()
                        .mt_4()
                        .mb_1p5()
                        .text_size(px(24.0))
                        .font_bold()
                        .line_height(relative(1.35))
                        .text_color(t.ink),
                    _ => div()
                        .mt_3()
                        .mb_1()
                        .text_size(px(20.0))
                        .font_semibold()
                        .line_height(relative(1.4))
                        .text_color(t.ink),
                };

                let mut container = heading_el.id(SharedString::from(format!("block-h-{}", block_id)));
                if let Some(state) = maybe_input_state {
                    container = container.child(
                        Input::new(&state)
                            .bordered(false)
                            .w_full(),
                    );
                } else if text_clone.trim().is_empty() {
                    container = container.child(render_inline_spans("Heading...", t).into_any_element());
                } else {
                    container = container.child(render_inline_spans(&text_clone, t).into_any_element());
                }
                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                container
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "backspace" {
                            this.handle_block_backspace(&bid_bs, window, cx);
                        } else if event.keystroke.key == "enter" {
                            this.handle_block_enter(&bid_enter, window, cx);
                        } else if event.keystroke.key == "up" {
                            this.handle_block_arrow(&bid_arr, true, window, cx);
                        } else if event.keystroke.key == "down" {
                            this.handle_block_arrow(&bid_arr, false, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            BlockKind::Bullet { text, indent } => {
                let text_clone = text.clone();
                let indent_px = px(24.0 * (*indent as f32));

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .w_full()
                        .into_any_element()
                } else {
                    div()
                        .text_size(px(16.0))
                        .line_height(relative(1.6))
                        .text_color(t.ink)
                        .child(render_inline_spans(&text_clone, t))
                        .into_any_element()
                };

                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                div()
                    .id(SharedString::from(format!("block-bullet-{}", block_id)))
                    .flex()
                    .items_start()
                    .gap_3()
                    .my_0p5()
                    .pl(indent_px)
                    .child(
                        div()
                            .w(px(6.0))
                            .h(px(6.0))
                            .mt(px(10.0))
                            .rounded_full()
                            .bg(t.ink),
                    )
                    .child(div().flex_1().child(content_el))
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "tab" {
                            this.handle_block_tab(&bid_arr, event.keystroke.modifiers.shift, cx);
                        } else if event.keystroke.key == "backspace" {
                            this.handle_block_backspace(&bid_bs, window, cx);
                        } else if event.keystroke.key == "enter" {
                            this.handle_block_enter(&bid_enter, window, cx);
                        } else if event.keystroke.key == "up" {
                            this.handle_block_arrow(&bid_arr, true, window, cx);
                        } else if event.keystroke.key == "down" {
                            this.handle_block_arrow(&bid_arr, false, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            BlockKind::Numbered { text, indent } => {
                let text_clone = text.clone();
                let indent_px = px(24.0 * (*indent as f32));
                let display_num = format!("{}.", idx + 1);

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .w_full()
                        .into_any_element()
                } else {
                    div()
                        .text_size(px(16.0))
                        .line_height(relative(1.6))
                        .text_color(t.ink)
                        .child(render_inline_spans(&text_clone, t))
                        .into_any_element()
                };

                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                div()
                    .id(SharedString::from(format!("block-num-{}", block_id)))
                    .flex()
                    .items_start()
                    .gap_2p5()
                    .my_0p5()
                    .pl(indent_px)
                    .child(
                        div()
                            .min_w(px(24.0))
                            .text_size(px(16.0))
                            .line_height(relative(1.6))
                            .font_semibold()
                            .text_color(t.ink_muted)
                            .child(display_num),
                    )
                    .child(div().flex_1().child(content_el))
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "tab" {
                            this.handle_block_tab(&bid_arr, event.keystroke.modifiers.shift, cx);
                        } else if event.keystroke.key == "backspace" {
                            this.handle_block_backspace(&bid_bs, window, cx);
                        } else if event.keystroke.key == "enter" {
                            this.handle_block_enter(&bid_enter, window, cx);
                        } else if event.keystroke.key == "up" {
                            this.handle_block_arrow(&bid_arr, true, window, cx);
                        } else if event.keystroke.key == "down" {
                            this.handle_block_arrow(&bid_arr, false, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            BlockKind::Todo { text, checked } => {
                let is_checked = *checked;
                let text_clone = text.clone();
                let target_id = block_id.clone();

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .w_full()
                        .into_any_element()
                } else {
                    div()
                        .text_size(px(16.0))
                        .line_height(relative(1.6))
                        .text_color(if is_checked { t.ink_muted } else { t.ink })
                        .when(is_checked, |s| s.line_through())
                        .child(render_inline_spans(&text_clone, t))
                        .into_any_element()
                };

                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                div()
                    .id(SharedString::from(format!("block-todo-{}", block_id)))
                    .flex()
                    .items_start()
                    .gap_3()
                    .my_1()
                    .child(
                        div()
                            .id(SharedString::from(format!("todo-box-{}", block_id)))
                            .cursor_pointer()
                            .mt(px(4.0))
                            .w(px(18.0))
                            .h(px(18.0))
                            .border_1()
                            .border_color(if is_checked { t.highlight } else { t.ink_muted })
                            .bg(if is_checked { t.highlight } else { t.bg })
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(move |s| s.opacity(0.85))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_block_todo(&target_id, cx);
                            }))
                            .when(is_checked, |this| {
                                this.child(
                                    Icon::new(IconName::Check)
                                        .size_3()
                                        .text_color(crate::theme::LIGHT_INK),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .when(is_checked, |s| s.opacity(0.6).line_through())
                            .child(content_el),
                    )
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "backspace" {
                            this.handle_block_backspace(&bid_bs, window, cx);
                        } else if event.keystroke.key == "enter" {
                            this.handle_block_enter(&bid_enter, window, cx);
                        } else if event.keystroke.key == "up" {
                            this.handle_block_arrow(&bid_arr, true, window, cx);
                        } else if event.keystroke.key == "down" {
                            this.handle_block_arrow(&bid_arr, false, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            BlockKind::Quote { text } => {
                let text_clone = text.clone();

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .w_full()
                        .into_any_element()
                } else {
                    div()
                        .text_size(px(16.0))
                        .line_height(relative(1.6))
                        .text_color(t.ink)
                        .child(render_inline_spans(&text_clone, t))
                        .into_any_element()
                };

                let bid_enter = block_id.clone();
                let bid_bs = block_id.clone();
                let bid_arr = block_id.clone();
                div()
                    .id(SharedString::from(format!("block-quote-{}", block_id)))
                    .my_2()
                    .pl_4()
                    .py_2()
                    .border_l(px(3.0))
                    .border_color(t.ink)
                    .bg(t.surface)
                    .rounded_r_sm()
                    .italic()
                    .child(content_el)
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "backspace" {
                            this.handle_block_backspace(&bid_bs, window, cx);
                        } else if event.keystroke.key == "enter" {
                            this.handle_block_enter(&bid_enter, window, cx);
                        } else if event.keystroke.key == "up" {
                            this.handle_block_arrow(&bid_arr, true, window, cx);
                        } else if event.keystroke.key == "down" {
                            this.handle_block_arrow(&bid_arr, false, window, cx);
                        }
                    }))
                    .into_any_element()
            }
            BlockKind::Code { language, text } => {
                let text_clone = text.clone();
                let lang_label = if language.is_empty() {
                    "code".to_string()
                } else {
                    language.to_lowercase()
                };

                div()
                    .id(SharedString::from(format!("block-code-{}", block_id)))
                    .my_3()
                    .rounded_md()
                    .border_1()
                    .border_color(t.ink_faint)
                    .bg(if t.is_dark {
                        hsla(0.0, 0.0, 0.07, 1.0)
                    } else {
                        hsla(54.0 / 360.0, 0.20, 0.74, 1.0)
                    })
                    .overflow_hidden()
                    .child(
                        // Header with language tag and copy button
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .border_b_1()
                            .border_color(t.ink_faint)
                            .child(
                                div()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(t.ink_muted)
                                    .child(lang_label),
                            )
                            .child(
                                Button::new(SharedString::from(format!("copy-code-{}", block_id)))
                                    .icon(IconName::Copy)
                                    .ghost()
                                    .small()
                                    .label("Copy")
                                    .tooltip("Copy code snippet")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(text_clone.clone()));
                                        this.status_message = Some(("Copied to clipboard".to_string(), Instant::now()));
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        if let Some(state) = maybe_input_state {
                            div()
                                .p_2()
                                .font_family(SharedString::from("Consolas, Menlo, Monaco, monospace"))
                                .text_size(px(14.0))
                                .line_height(relative(1.5))
                                .child(
                                    Input::new(&state)
                                        .bordered(false)
                                        .w_full(),
                                )
                                .into_any_element()
                        } else {
                            div()
                                .p_3()
                                .overflow_hidden()
                                .font_family(SharedString::from("Consolas, Menlo, Monaco, monospace"))
                                .text_size(px(14.0))
                                .line_height(relative(1.5))
                                .text_color(t.ink)
                                .child(text.clone())
                                .into_any_element()
                        },
                    )
                    .into_any_element()
            }
            BlockKind::Divider => {
                div()
                    .id(SharedString::from(format!("block-div-{}", block_id)))
                    .my_4()
                    .h_px()
                    .w_full()
                    .bg(t.ink_faint)
                    .into_any_element()
            }
            BlockKind::Unknown => {
                div()
                    .id(SharedString::from(format!("block-unknown-{}", block_id)))
                    .my_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_dashed()
                    .border_color(t.ink_faint)
                    .bg(t.surface)
                    .text_xs()
                    .text_color(t.ink_muted)
                    .child("Custom Block (preserved)")
                    .into_any_element()
            }
        }
    }

    pub fn toggle_block_todo(&mut self, block_id: &str, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                if let Some(block) = note.blocks.iter_mut().find(|b| b.id == block_id) {
                    if let BlockKind::Todo { checked, .. } = &mut block.kind {
                        *checked = !*checked;
                        note.updated_at = chrono::Local::now();
                        let _ = self.storage.save(note);
                        cx.notify();
                    }
                }
            }
        }
    }


}
