pub mod blocks;
pub mod inline;
pub mod starter;

use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, input::{Editor, Input}, Icon, IconName, Sizable};
use gpui_kit::prelude::FluentBuilder;
use crate::theme::{CurrentScreen, EditorModeView, LIGHT_INK};
use crate::NotesApp;

impl NotesApp {
    pub(crate) fn render_editor(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let active_note = self.selected_note().cloned();

        // Calculate word & character metrics for editor status bar
        let body_content = active_note.as_ref().map(|n| n.body.as_str()).unwrap_or("");
        let word_count = body_content.split_whitespace().count();
        let char_count = body_content.chars().count();
        let read_time_mins = (word_count / 200).max(1);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            .relative()
            // Top Nav: [< Back] [Segmented Mode: Visual | Source] [Zen Mode] [Theme] [Save] [Delete]
            .child(
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
                    // Segmented Editor Mode Switcher (Single Layout: Visual vs Source)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .p_0p5()
                            .rounded_md()
                            .bg(t.surface)
                            .border_1()
                            .border_color(t.ink_faint)
                            // Visual Mode Pill
                            .child(
                                div()
                                    .id("tab-visual")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .font_bold()
                                    .bg(if self.editor_mode == EditorModeView::Visual {
                                        t.bg
                                    } else {
                                        hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .text_color(if self.editor_mode == EditorModeView::Visual {
                                        t.ink
                                    } else {
                                        t.ink_muted
                                    })
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor_mode = EditorModeView::Visual;
                                        cx.notify();
                                    }))
                                    .child("👁 Visual"),
                            )
                            // Source Mode Pill (uses Editor)
                            .child(
                                div()
                                    .id("tab-source")
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .font_bold()
                                    .bg(if self.editor_mode == EditorModeView::Source {
                                        t.bg
                                    } else {
                                        hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .text_color(if self.editor_mode == EditorModeView::Source {
                                        t.ink
                                    } else {
                                        t.ink_muted
                                    })
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor_mode = EditorModeView::Source;
                                        cx.notify();
                                    }))
                                    .child("✎ Source"),
                            ),
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
                                    .icon(if self.zen_focus_mode {
                                        IconName::Maximize
                                    } else {
                                        IconName::Minimize
                                    })
                                    .ghost()
                                    .small()
                                    .tooltip(if self.zen_focus_mode {
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
                    ),
            )
            // Notion-Style Quick Formatting Strip (hides in Zen focus mode)
            .when(!self.zen_focus_mode, |this| {
                this.child(
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
                        ),
                )
            })
            // Single Document Canvas (WYSIWYG Live Canvas or Source Editor)
            .child(
                div()
                    .id("editor-canvas-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .when(!self.zen_focus_mode, |this| this.px_10().py_6())
                    .when(self.zen_focus_mode, |this| this.px_20().py_8())
                    // Date & Status Header
                    .when(active_note.is_some(), |this| {
                        let note = active_note.as_ref().unwrap();
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .mb_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(t.ink_muted)
                                                .child(note.formatted_full_date()),
                                        )
                                        .child(div().w_px().h(px(10.0)).bg(t.ink_faint))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(t.ink_muted)
                                                .child(format!("{} min read", read_time_mins)),
                                        ),
                                )
                                .when(self.status_message.is_some(), |this| {
                                    let (msg, _) = self.status_message.as_ref().unwrap();
                                    this.child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_sm()
                                            .bg(t.highlight)
                                            .text_xs()
                                            .font_bold()
                                            .text_color(LIGHT_INK)
                                            .child(format!("✓ {}", msg)),
                                    )
                                }),
                        )
                    })
                    // Title row using Input (Single-line value control)
                    .child(
                        div()
                            .mb_2()
                            .child(
                                Input::new(&self.title_state)
                                    .bordered(false)
                                    .large(),
                            ),
                    )
                    // Ruled horizontal line under title
                    .child(
                        div()
                            .h_px()
                            .bg(t.ink)
                            .mb_4(),
                    )
                    // Structured Tasks Section (if note has checklist items)
                    .when(active_note.as_ref().map_or(false, |n| !n.checklist.is_empty()), |this| {
                        let note = active_note.as_ref().unwrap();
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .mb_4()
                                .p_3()
                                .rounded_md()
                                .bg(t.surface)
                                .border_1()
                                .border_color(t.ink_faint)
                                .children(note.checklist.iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let is_checked = item.checked;
                                    let text = item.text.clone();
                                    let del_id = item_id.clone();

                                    div()
                                        .id(SharedString::from(format!("chk-row-{}", item_id)))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap_3()
                                        .child(
                                            div()
                                                .id(SharedString::from(format!("chk-toggle-{}", item_id)))
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .cursor_pointer()
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.toggle_checklist_item(&item_id, cx);
                                                }))
                                                .child(
                                                    div()
                                                        .w(px(16.0))
                                                        .h(px(16.0))
                                                        .border_1()
                                                        .border_color(t.ink)
                                                        .rounded_sm()
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .when(is_checked, |this| {
                                                            this.child(
                                                                Icon::new(IconName::Check)
                                                                    .size_3()
                                                                    .text_color(t.ink),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(t.ink)
                                                        .when(is_checked, |this| {
                                                            this.text_color(t.ink_muted).line_through()
                                                        })
                                                        .child(text),
                                                ),
                                        )
                                        .child(
                                            Button::new(SharedString::from(format!("del-chk-{}", del_id)))
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .tooltip("Remove task")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.remove_checklist_item(&del_id, cx);
                                                })),
                                        )
                                })),
                        )
                    })
                    // Single Layout Document Workspace: WYSIWYG Visual Mode OR Source Editor Mode
                    .child(
                        match self.editor_mode {
                            EditorModeView::Visual => {
                                if body_content.trim().is_empty() {
                                    self.render_notion_empty_starter(&t, cx).into_any_element()
                                } else {
                                    div()
                                        .flex_1()
                                        .mb_4()
                                        .child(self.render_notion_blocks(body_content, &t, cx))
                                        .into_any_element()
                                }
                            }
                            EditorModeView::Source => {
                                div()
                                    .flex_1()
                                    .mb_4()
                                    .min_h(px(380.0))
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        Editor::new(&self.editor_state)
                                            .bordered(false)
                                            .h(px(380.0)),
                                    )
                                    .into_any_element()
                            }
                        },
                    )
                    // Visual Mode Bottom Action Strip: Quick adders and switch to source
                    .when(self.editor_mode == EditorModeView::Visual && !body_content.trim().is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pt_4()
                                .pb_2()
                                .border_t_1()
                                .border_color(t.ink_faint)
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Button::new("btn-bottom-add-task")
                                                .ghost()
                                                .small()
                                                .icon(IconName::Check)
                                                .label("+ Task")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("checklist", window, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-bottom-add-heading")
                                                .ghost()
                                                .small()
                                                .label("+ Heading")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("h2", window, cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-bottom-add-code")
                                                .ghost()
                                                .small()
                                                .icon(IconName::Copy)
                                                .label("+ Code")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.insert_formatting("code", window, cx);
                                                })),
                                        ),
                                )
                                .child(
                                    Button::new("btn-switch-to-source")
                                        .ghost()
                                        .small()
                                        .label("Edit Source ✎")
                                        .tooltip("Switch to raw Markdown Editor")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.editor_mode = EditorModeView::Source;
                                            cx.notify();
                                        })),
                                ),
                        )
                    })
                    // Attached images section
                    .when(active_note.as_ref().map_or(false, |n| !n.images.is_empty()), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .my_4()
                                .p_3()
                                .border_1()
                                .border_color(t.ink_faint)
                                .bg(t.surface)
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(140.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(t.bg)
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .gap_2()
                                                .child(Icon::new(IconName::FileText).size_8().text_color(t.ink))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .italic()
                                                        .text_color(t.ink_muted)
                                                        .child("Zen Photo Attachment"),
                                                ),
                                        ),
                                ),
                        )
                    })
                    // Voice player bar (as in inspo wireframes)
                    .when(active_note.as_ref().map_or(false, |n| n.voice.is_some()), |this| {
                        let voice = active_note.as_ref().unwrap().voice.as_ref().unwrap();
                        let is_playing = voice.is_playing;

                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_2()
                                .my_3()
                                .border_1()
                                .border_color(t.ink)
                                .bg(t.bg)
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .child(format!("00:00:{:02}", voice.current_seconds)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .mx_4()
                                        .h(px(2.0))
                                        .bg(t.ink)
                                        .relative()
                                        .child(
                                            div()
                                                .absolute()
                                                .left(px(40.0))
                                                .top(px(-3.0))
                                                .w(px(8.0))
                                                .h(px(8.0))
                                                .rounded_full()
                                                .bg(t.ink),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Button::new("btn-voice-play-pause")
                                                .icon(if is_playing {
                                                    IconName::Play
                                                } else {
                                                    IconName::ChevronRight
                                                })
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.toggle_voice_player(cx);
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-voice-close")
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.remove_voice_recording(cx);
                                                })),
                                        ),
                                ),
                        )
                    }),
            )
            // Streamlined Editor Status Footer: Word count, character count, and quick task adder
            .child(
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
                    ),
            )
            // Add Modal / Drawer (Images & Voice Recordings)
            .when(self.show_add_modal, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(hsla(0.0, 0.0, 0.0, 0.35))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .w(px(320.0))
                                .p_6()
                                .bg(t.bg)
                                .border_2()
                                .border_color(t.ink)
                                .shadow_xl()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_bold()
                                                .text_color(t.ink)
                                                .child("Attach Media"),
                                        )
                                        .child(
                                            Button::new("btn-close-modal")
                                                .icon(IconName::Close)
                                                .ghost()
                                                .small()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_add_modal = false;
                                                    cx.notify();
                                                })),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("btn-modal-add-image")
                                        .cursor_pointer()
                                        .p_3()
                                        .border_1()
                                        .border_color(t.ink)
                                        .text_center()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .hover(move |s| s.bg(t.bg_hover))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.add_sample_image(cx);
                                        }))
                                        .child("Add image"),
                                )
                                .child(
                                    div()
                                        .id("btn-modal-add-voice")
                                        .cursor_pointer()
                                        .p_3()
                                        .border_1()
                                        .border_color(t.ink)
                                        .text_center()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(t.ink)
                                        .hover(move |s| s.bg(t.bg_hover))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.add_sample_voice_recording(cx);
                                        }))
                                        .child("Add voice recording"),
                                ),
                        ),
                )
            })
    }
}
