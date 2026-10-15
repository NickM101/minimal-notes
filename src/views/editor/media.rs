use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, Icon, IconName, Sizable};
use crate::models::VoiceRecording;
use crate::theme::AppTheme;
use crate::NotesApp;

pub fn render_image_attachment(t: &AppTheme) -> impl IntoElement {
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
        )
}

pub fn render_voice_player(voice: &VoiceRecording, t: &AppTheme, cx: &mut Context<NotesApp>) -> impl IntoElement {
    let is_playing = voice.is_playing;

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
        )
}
