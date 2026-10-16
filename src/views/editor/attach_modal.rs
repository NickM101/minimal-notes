use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    IconName, Sizable,
};
use gpui_kit::*;

pub fn render_attach_modal(
    _this: &NotesApp,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
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
        )
}
