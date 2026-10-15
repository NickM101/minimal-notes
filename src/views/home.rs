use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, Icon, IconName};
use crate::theme::CurrentScreen;
use crate::NotesApp;

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
                            .gap_3()
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
                            )
                            // Search Button
                            .child(
                                Button::new("btn-home-search")
                                    .icon(IconName::Search)
                                    .ghost()
                                    .tooltip("Browse & Search Notes")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.current_screen = CurrentScreen::Browse;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Center Area: Balanced composition with breathing room
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
                    // Balanced Action Group at the lower third
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_4()
                            .child(
                                // Boxed [ New Note ] Button
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
                            )
                            .child(
                                div()
                                    .id("btn-cover-browse-notes")
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .px_4()
                                    .py_1p5()
                                    .rounded_full()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(t.ink_muted)
                                    .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.current_screen = CurrentScreen::Browse;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Search).size_3p5())
                                    .child("Browse Notes"),
                            ),
                    ),
            )

            // Bottom Right Sign In / Profile docked at the very bottom right
            .child(
                div()
                    .absolute()
                    .right(px(32.0))
                    .bottom(px(32.0))
                    .child(self.render_avatar_button(&t, cx)),
            )



    }


}
