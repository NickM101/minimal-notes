use crate::theme::CurrentScreen;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    IconName,
};
use gpui_kit::*;

impl NotesApp {
    pub(crate) fn render_home(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(t.bg)
            .text_color(t.ink)
            // Top Nav
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_8()
                    .pt_6()
                    .child(self.render_logo(&t))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            // Dark / Light Mode Toggle Button
                            .child(
                                Button::new("btn-home-theme-toggle")
                                    .icon(if t.is_dark {
                                        IconName::Sun
                                    } else {
                                        IconName::Moon
                                    })
                                    .ghost()
                                    .tooltip(if t.is_dark {
                                        "Switch to Light Mode (Old Paper)"
                                    } else {
                                        "Switch to Dark Mode (Pitch Black #000)"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_theme(cx);
                                    })),
                            ),
                    ),
            )
            // Center Area: Hanging Zen Lamp & New Note Button
            .child(
                div()
                    .flex()
                    .flex_1()
                    .flex_col()
                    .items_center()
                    .justify_between()
                    .pt_4()
                    .pb_12()
                    // Hanging Zen Lamp
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.render_zen_lamp(&t)),
                    )
                    // Boxed [ New Note ] Button centered in lower third
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .id("btn-cover-new-note")
                                    .cursor_pointer()
                                    .px_10()
                                    .py_3()
                                    .rounded_md()
                                    .border_2()
                                    .border_color(t.ink)
                                    .bg(t.bg)
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .active(move |s| s.bg(t.bg_active))
                                    .text_sm()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_new_note(window, cx);
                                    }))
                                    .child("New Note"),
                            ),
                    ),
            )
            // Bottom Bar: Browse Notes docked at bottom left, Sign In / Profile docked at bottom right
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_8()
                    .pb_8()
                    .child(
                        div()
                            .id("btn-cover-browse-notes")
                            .cursor_pointer()
                            .text_sm()
                            .font_bold()
                            .text_color(t.ink)
                            .hover(|s| s.underline())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_screen = CurrentScreen::Browse;
                                cx.notify();
                            }))
                            .child("Browse Notes"),
                    )
                    .child(self.render_avatar_button(&t, cx)),
            )
    }
}
