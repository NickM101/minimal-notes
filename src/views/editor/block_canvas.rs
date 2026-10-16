use crate::app::editor_reducer::EditorAction;
use crate::models::{Block, BlockKind};
use crate::theme::AppTheme;
use crate::views::editor::inline::render_inline_spans;
use crate::NotesApp;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Backspace, Escape, IndentInline, Input, MoveDown, MoveUp, OutdentInline},
    Icon, Sizable,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::time::Instant;

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
        let slash_query = self
            .slash_menu_target
            .as_ref()
            .map(|(_, q)| q.to_lowercase())
            .unwrap_or_default();
        let slash_selection = self.slash_menu_selection;

        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .py_2()
            .w_full()
            .relative()
            // Block-level key handling, ported from cascade-editor's action model:
            // the focused Input consumes keys it owns; only these actions ever
            // reach the canvas. Backspace-at-start propagates from the input by
            // design (merge hook); up/down/tab are unhandled by single-line inputs.
            .on_action(cx.listener(move |this, _: &Backspace, window, cx| {
                if let Some(id) = this.focused_block_id.clone() {
                    this.dispatch(EditorAction::MergeWithPrevious { id }, window, cx);
                }
            }))
            .on_action(cx.listener(move |this, _: &MoveUp, _window, cx| {
                // Slash menu open? Arrow keys navigate the menu, not the document.
                if this.slash_menu_target.is_some() {
                    this.slash_nav(true, cx);
                } else {
                    this.focus_adjacent_block(true, _window, cx);
                }
            }))
            .on_action(cx.listener(move |this, _: &MoveDown, _window, cx| {
                if this.slash_menu_target.is_some() {
                    this.slash_nav(false, cx);
                } else {
                    this.focus_adjacent_block(false, _window, cx);
                }
            }))
            .on_action(cx.listener(move |this, _: &IndentInline, window, cx| {
                if let Some(id) = this.focused_block_id.clone() {
                    this.dispatch(EditorAction::Indent { id, delta: 1 }, window, cx);
                }
            }))
            .on_action(cx.listener(move |this, _: &OutdentInline, window, cx| {
                if let Some(id) = this.focused_block_id.clone() {
                    this.dispatch(EditorAction::Indent { id, delta: -1 }, window, cx);
                }
            }))
            .on_action(cx.listener(move |this, _: &Escape, _window, cx| {
                this.slash_close(cx);
            }))
            .children({
                let mut num_counter: usize = 0;
                blocks
                    .iter()
                    .enumerate()
                    .map(move |(idx, block)| {
                        // Fix 8: track numbered list sequence, reset on any non-Numbered block
                        match &block.kind {
                            crate::models::BlockKind::Numbered { .. } => {
                                num_counter += 1;
                            }
                            _ => {
                                num_counter = 0;
                            }
                        }
                        let display_num = num_counter;

                        let block_id = block.id.clone();
                        let is_hovered = self.hovered_block_id.as_deref() == Some(&block_id);
                        let is_this_slash_target = slash_target_id.as_deref() == Some(&block_id);
                        let bid_hover = block_id.clone();
                        let bid_action = block_id.clone();
                        let bid_click = block_id.clone(); // Fix 1

                        div()
                            .id(SharedString::from(format!("block-row-{}", block_id)))
                            .flex()
                            .items_start()
                            .relative()
                            .w_full()
                            .group("block-row")
                            // No focus border: a border toggling on focus shifted layout by 2px
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
                            // Fix 1: click the row → focus this block's input
                            .on_click(cx.listener(move |this, _, window, cx| {
                                // Keep the click from bubbling to the canvas (which focuses the last block)
                                cx.stop_propagation();
                                if let Some(handle) = this
                                    .block_input_states
                                    .iter()
                                    .find(|h| h.block_id == bid_click)
                                {
                                    let state = handle.state.clone();
                                    state.update(cx, |s, cx| {
                                        s.focus(window, cx);
                                    });
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
                                            .id(SharedString::from(format!(
                                                "gutter-btn-{}",
                                                block_id
                                            )))
                                            .cursor_pointer()
                                            .p_0p5()
                                            .rounded_sm()
                                            .hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.15)))
                                            // Drag-handle grip affordance
                                            .child(
                                                Icon::new(IconName::GripVertical)
                                                    .size_4()
                                                    .text_color(t.ink_muted),
                                            )
                                            .on_click(cx.listener(move |this, _, _window, cx| {
                                                // Gutter click opens the slash menu on this block!
                                                this.slash_menu_target =
                                                    Some((bid_action.clone(), String::new()));
                                                this.slash_menu_selection = Some(0);
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // Block content
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_read_only_block(
                                        idx,
                                        display_num,
                                        block,
                                        t,
                                        cx,
                                    ))
                                    // Inline Slash Command Popover Menu directly under this block!
                                    .when(is_this_slash_target, |this| {
                                        this.child(
                                            crate::views::editor::slash_menu::render_slash_menu(
                                                &block_id,
                                                &slash_query,
                                                slash_selection,
                                                t,
                                                cx,
                                            ),
                                        )
                                    }),
                            )
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
    }

    fn render_read_only_block(
        &self,
        _idx: usize,
        display_num: usize, // Fix 8: sequential number within its list run
        block: &Block,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let block_id = block.id.clone();

        let maybe_input_state = self
            .block_input_states
            .iter()
            .find(|h| h.block_id == block_id)
            .map(|h| h.state.clone());

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
                            .appearance(false)
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

                let mut container =
                    heading_el.id(SharedString::from(format!("block-h-{}", block_id)));
                if let Some(state) = maybe_input_state {
                    container = container.child(
                        Input::new(&state)
                            .bordered(false)
                            .appearance(false)
                            .w_full(),
                    );
                } else if text_clone.trim().is_empty() {
                    container =
                        container.child(render_inline_spans("Heading...", t).into_any_element());
                } else {
                    container =
                        container.child(render_inline_spans(&text_clone, t).into_any_element());
                }
                container.into_any_element()
            }
            BlockKind::Bullet { text, indent } => {
                let text_clone = text.clone();
                let indent_px = px(24.0 * (*indent as f32));

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .appearance(false)
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
                    .into_any_element()
            }
            BlockKind::Numbered { text, indent } => {
                let text_clone = text.clone();
                let indent_px = px(24.0 * (*indent as f32));
                // display_num is passed in from the parent iterator (Fix 8)

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .appearance(false)
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
                            // Fix 8: use display_num (counted within the consecutive run)
                            .child(format!("{}.", display_num)),
                    )
                    .child(div().flex_1().child(content_el))
                    .into_any_element()
            }
            BlockKind::Todo { text, checked } => {
                let is_checked = *checked;
                let text_clone = text.clone();
                let target_id = block_id.clone();

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .appearance(false)
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
                    .into_any_element()
            }
            BlockKind::Quote { text } => {
                let text_clone = text.clone();

                let content_el: AnyElement = if let Some(state) = maybe_input_state {
                    Input::new(&state)
                        .bordered(false)
                        .appearance(false)
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
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            text_clone.clone(),
                                        ));
                                        this.status_message = Some((
                                            "Copied to clipboard".to_string(),
                                            Instant::now(),
                                        ));
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(if let Some(state) = maybe_input_state {
                        div()
                            .p_2()
                            .font_family(SharedString::from("Consolas, Menlo, Monaco, monospace"))
                            .text_size(px(14.0))
                            .line_height(relative(1.5))
                            .child(
                                Input::new(&state)
                                    .bordered(false)
                                    .appearance(false)
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
                    })
                    .into_any_element()
            }
            BlockKind::Divider => div()
                .id(SharedString::from(format!("block-div-{}", block_id)))
                .my_4()
                .h_px()
                .w_full()
                .bg(t.ink_faint)
                .into_any_element(),
            BlockKind::Unknown => div()
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
                .into_any_element(),
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
