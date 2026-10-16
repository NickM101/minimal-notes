use crate::theme::{AppTheme, CurrentScreen};
use crate::NotesApp;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Sizable,
};
use gpui_kit::*;

pub fn render_top_nav(
    this: &NotesApp,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .px_6()
        .py_3()
        .border_b_1()
        .border_color(t.ink_faint)
        .child(
            Button::new("btn-editor-back")
                .icon(IconName::ChevronLeft)
                .ghost()
                .tooltip("Back to Notes (Esc)")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.current_screen = CurrentScreen::Browse;
                    cx.notify();
                })),
        )
        // Right Toolbar Controls
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                // Zen Focus Mode Toggle
                .child(
                    Button::new("btn-editor-zen")
                        .icon(if this.zen_focus_mode {
                            IconName::Maximize
                        } else {
                            IconName::Minimize
                        })
                        .ghost()
                        .small()
                        .tooltip(if this.zen_focus_mode {
                            "Exit Zen Focus"
                        } else {
                            "Zen Focus Mode"
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.zen_focus_mode = !this.zen_focus_mode;
                            cx.notify();
                        })),
                )
                // Light / Dark Mode Toggle
                .child(
                    Button::new("btn-editor-theme-toggle")
                        .icon(if t.is_dark {
                            IconName::Sun
                        } else {
                            IconName::Moon
                        })
                        .ghost()
                        .small()
                        .tooltip(if t.is_dark { "Light mode" } else { "Dark mode" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_theme(cx);
                        })),
                )
                // Add Media Button
                .child(
                    div()
                        .id("btn-editor-add")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .font_bold()
                        .text_color(t.ink)
                        .hover(|s| s.underline())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_add_modal = !this.show_add_modal;
                            cx.notify();
                        }))
                        .child("Attach"),
                )
                // Save Button
                .child(
                    Button::new("btn-editor-save")
                        .icon(IconName::Check)
                        .ghost()
                        .small()
                        .tooltip("Save Note")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.trigger_save(cx);
                        })),
                )
                // Delete Button
                .child(
                    Button::new("btn-editor-delete")
                        .icon(IconName::Delete)
                        .ghost()
                        .small()
                        .tooltip("Delete Note")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.delete_selected_note(window, cx);
                        })),
                ),
        )
}

pub fn render_formatting_strip(
    _this: &NotesApp,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    // Keep the strip minimal: only block-level transforms that actually work.
    // Inline styles (bold/italic/highlight) are typed as markdown instead.
    div()
        .flex()
        .items_center()
        .gap_1()
        .px_6()
        .py_1p5()
        .border_b_1()
        .border_color(t.ink_faint)
        .bg(t.surface)
        .child(
            Button::new("fmt-h1")
                .ghost()
                .small()
                .label("H1")
                .tooltip("Heading 1 (#)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("h1", window, cx);
                })),
        )
        .child(
            Button::new("fmt-h2")
                .ghost()
                .small()
                .label("H2")
                .tooltip("Heading 2 (##)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("h2", window, cx);
                })),
        )
        .child(
            Button::new("fmt-h3")
                .ghost()
                .small()
                .label("H3")
                .tooltip("Heading 3 (###)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("h3", window, cx);
                })),
        )
        .child(div().w_px().h(px(14.0)).bg(t.ink_faint).mx_1())
        .child(
            Button::new("fmt-bullet")
                .icon(IconName::List)
                .ghost()
                .small()
                .tooltip("Bullet List (- item)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("bullet", window, cx);
                })),
        )
        .child(
            Button::new("fmt-number")
                .icon(IconName::ListOrdered)
                .ghost()
                .small()
                .tooltip("Numbered List (1. item)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("number", window, cx);
                })),
        )
        .child(
            Button::new("fmt-task")
                .icon(IconName::ListTodo)
                .ghost()
                .small()
                .tooltip("To-do (- [ ])")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("checklist", window, cx);
                })),
        )
        .child(
            Button::new("fmt-quote")
                .icon(IconName::TextQuote)
                .ghost()
                .small()
                .tooltip("Blockquote (> quote)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("quote", window, cx);
                })),
        )
        .child(
            Button::new("fmt-code-block")
                .icon(IconName::SquareCode)
                .ghost()
                .small()
                .tooltip("Code Block (```lang)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("code", window, cx);
                })),
        )
        .child(
            Button::new("fmt-divider")
                .icon(IconName::Minus)
                .ghost()
                .small()
                .tooltip("Divider (---)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("divider", window, cx);
                })),
        )
}

pub fn render_editor_footer(
    _this: &NotesApp,
    t: &AppTheme,
    word_count: usize,
    char_count: usize,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .px_6()
        .py_2()
        .border_t_1()
        .border_color(t.ink_faint)
        .bg(t.bg)
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .text_xs()
                        .text_color(t.ink_muted)
                        .child(format!("{} words", word_count)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(t.ink_muted)
                        .child(format!("{} chars", char_count)),
                ),
        )
}
