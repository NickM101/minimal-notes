use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, IconName, Sizable};
use crate::theme::{AppTheme, CurrentScreen, LIGHT_INK};
use crate::NotesApp;

pub fn render_top_nav(this: &NotesApp, t: &AppTheme, cx: &mut Context<NotesApp>) -> impl IntoElement {
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

pub fn render_formatting_strip(_this: &NotesApp, t: &AppTheme, cx: &mut Context<NotesApp>) -> impl IntoElement {
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
            Button::new("fmt-bold")
                .ghost()
                .small()
                .label("B")
                .tooltip("Bold (**text**)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("bold", window, cx);
                })),
        )
        .child(
            Button::new("fmt-italic")
                .ghost()
                .small()
                .label("I")
                .tooltip("Italic (*text*)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("italic", window, cx);
                })),
        )
        .child(
            div()
                .id("fmt-highlight")
                .cursor_pointer()
                .w(px(20.0))
                .h(px(20.0))
                .bg(t.highlight)
                .rounded_sm()
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .font_bold()
                .text_color(LIGHT_INK)
                .hover(|s| s.opacity(0.8))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("highlight", window, cx);
                }))
                .child("T"),
        )
        .child(
            Button::new("fmt-strike")
                .ghost()
                .small()
                .label("~")
                .tooltip("Strikethrough (~~text~~)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("strike", window, cx);
                })),
        )
        .child(
            Button::new("fmt-code-inline")
                .ghost()
                .small()
                .label("`")
                .tooltip("Inline Code (`code`)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("code_inline", window, cx);
                })),
        )
        .child(div().w_px().h(px(14.0)).bg(t.ink_faint).mx_1())
        .child(
            Button::new("fmt-task")
                .icon(IconName::Check)
                .ghost()
                .small()
                .tooltip("Interactive Task (- [ ])")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("checklist", window, cx);
                })),
        )
        .child(
            Button::new("fmt-bullet")
                .icon(IconName::Menu)
                .ghost()
                .small()
                .tooltip("Bullet List (- item)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("bullet", window, cx);
                })),
        )
        .child(
            Button::new("fmt-number")
                .ghost()
                .small()
                .label("1.")
                .tooltip("Numbered List (1. item)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("number", window, cx);
                })),
        )
        .child(
            Button::new("fmt-quote")
                .ghost()
                .small()
                .label("\"")
                .tooltip("Blockquote (> quote)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("quote", window, cx);
                })),
        )
        .child(
            Button::new("fmt-callout")
                .icon(IconName::Info)
                .ghost()
                .small()
                .tooltip("Callout Box (> [!NOTE])")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("callout", window, cx);
                })),
        )
        .child(
            Button::new("fmt-code-block")
                .icon(IconName::Copy)
                .ghost()
                .small()
                .tooltip("Code Block (```lang)")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.insert_formatting("code", window, cx);
                })),
        )
        .child(
            Button::new("fmt-divider")
                .ghost()
                .small()
                .label("—")
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
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
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
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    Button::new("btn-footer-settings")
                        .icon(IconName::Settings)
                        .ghost()
                        .small()
                        .tooltip("App Settings")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_settings_modal = true;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("btn-footer-update")
                        .icon(IconName::RefreshCw)
                        .ghost()
                        .small()
                        .tooltip("Check for Updates")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_update_modal = true;
                            this.is_checking_update = false;
                            this.update_checked = false;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("btn-footer-add-task")
                        .icon(IconName::Plus)
                        .ghost()
                        .small()
                        .label("Task")
                        .tooltip("Add Checklist Item")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.add_checklist_item(cx);
                        })),
                ),
        )
}
