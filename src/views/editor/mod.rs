pub mod attach_modal;
pub mod block_canvas;
pub mod blocks;
pub mod inline;
pub mod media;
pub mod slash_menu;
pub mod starter;
pub mod toolbar;

use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{input::Input, Sizable};
use gpui_kit::prelude::FluentBuilder;
use crate::theme::LIGHT_INK;
use crate::NotesApp;

impl NotesApp {
    pub(crate) fn render_editor(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let active_note = self.selected_note().cloned();

        // Calculate word & character metrics for editor status bar
        let plain_body = active_note.as_ref().map(|n| n.plain_text()).unwrap_or_default();
        let body_content = plain_body.as_str();
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
            .child(toolbar::render_top_nav(self, &t, cx))
            // Quick Formatting Strip (hides in Zen focus mode)
            .when(!self.zen_focus_mode, |this| {
                this.child(toolbar::render_formatting_strip(self, &t, cx))
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
                    // Centered Single Column Document Canvas (max-w 720px)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .w_full()
                                    .max_w(px(720.0))
                                    .flex()
                                    .flex_col()
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
                                    // Title row using borderless Input
                                    .child(
                                        div()
                                            .mb_2()
                                            .child(
                                                Input::new(&self.title_state)
                                                    .bordered(false)
                                                    .large(),
                                            )
                                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                                                if event.keystroke.key == "down" || event.keystroke.key == "enter" {
                                                    if let Some((_, first_state)) = this.block_input_states.first() {
                                                        first_state.update(cx, |s, cx| {
                                                             s.focus(window, cx);
                                                        });
                                                        cx.notify();
                                                    }
                                                }
                                            })),
                                    )
                                    // Ruled horizontal line under title
                                    .child(
                                        div()
                                            .h_px()
                                            .bg(t.ink)
                                            .mb_4(),
                                    )
                                    // Structured block canvas
                                    .child(
                                        if let Some(note) = active_note.as_ref() {
                                            if note.blocks.is_empty() {
                                                self.render_notion_empty_starter(&t, cx).into_any_element()
                                            } else {
                                                div()
                                                    .flex_1()
                                                    .mb_6()
                                                    .child(self.render_block_canvas(&note.blocks, &t, cx))
                                                    .into_any_element()
                                             }
                                        } else {
                                            div().into_any_element()
                                        },
                                    )
                                    // Attached images section
                                    .when(active_note.as_ref().map_or(false, |n| !n.images.is_empty()), |this| {
                                        this.child(media::render_image_attachment(&t))
                                    })
                                    // Voice player bar
                                    .when(active_note.as_ref().map_or(false, |n| n.voice.is_some()), |this| {
                                        let voice = active_note.as_ref().unwrap().voice.as_ref().unwrap();
                                        this.child(media::render_voice_player(voice, &t, cx))
                                    }),
                            ),
                    ),
            )
            // Streamlined Editor Status Footer
            .child(toolbar::render_editor_footer(self, &t, word_count, char_count, cx))
            // Add Modal / Drawer (Images & Voice Recordings)
            .when(self.show_add_modal, |this| {
                this.child(attach_modal::render_attach_modal(self, &t, cx))
            })
    }
}
