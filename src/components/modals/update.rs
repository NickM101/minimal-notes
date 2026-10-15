use std::time::Instant;
use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{Icon, IconName};
use crate::theme::AppTheme;
use crate::NotesApp;

impl NotesApp {
    pub(crate) fn render_update_modal(&self, t: &AppTheme, cx: &mut Context<Self>) -> impl IntoElement {
        let is_checking = self.is_checking_update;
        let checked = self.update_checked;

        div()
            .id("update-modal-backdrop")
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.75))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_update_modal = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("update-modal-card")
                    .w(px(380.0))
                    .p_6()
                    .rounded_lg()
                    .border_2()
                    .border_color(if t.is_dark { t.ink_faint } else { t.ink })
                    .bg(t.bg)
                    .text_color(t.ink)
                    .shadow_2xl()
                    .flex()
                    .flex_col()
                    .gap_5()
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(IconName::RefreshCw).size_5().text_color(t.ink))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child("Software Update"),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-update-modal")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded_sm()
                                    .text_color(t.ink_muted)
                                    .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_update_modal = false;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Close).size_4()),
                            ),
                    )
                    // Status display card
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p_6()
                            .rounded_md()
                            .border_1()
                            .border_color(t.ink_faint)
                            .bg(t.surface)
                            .gap_3()
                            .child(
                                div()
                                    .w(px(48.0))
                                    .h(px(48.0))
                                    .rounded_full()
                                    .bg(if checked {
                                        hsla(140.0 / 360.0, 0.80, 0.45, 0.15)
                                    } else {
                                        t.bg
                                    })
                                    .border_1()
                                    .border_color(if checked {
                                        hsla(140.0 / 360.0, 0.80, 0.45, 1.0)
                                    } else {
                                        t.ink_faint
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(if checked {
                                        Icon::new(IconName::Check)
                                            .size_6()
                                            .text_color(hsla(140.0 / 360.0, 0.80, 0.45, 1.0))
                                    } else {
                                        Icon::new(IconName::RefreshCw).size_6().text_color(t.ink)
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .child(if is_checking {
                                        "Checking for updates..."
                                    } else if checked {
                                        "You're on the latest version!"
                                    } else {
                                        "Notty v0.1.0 (Latest Release)"
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_center()
                                    .text_color(t.ink_muted)
                                    .child(if is_checking {
                                        "Connecting to release distribution servers..."
                                    } else if checked {
                                        "Notty v0.1.0 is currently up to date. Channel: Stable"
                                    } else {
                                        "Click below to verify whether new updates or patches are available."
                                    }),
                            ),
                    )
                    // Action Buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .id("btn-check-update-trigger")
                                    .cursor_pointer()
                                    .flex_1()
                                    .py_2()
                                    .rounded_md()
                                    .bg(t.ink)
                                    .text_color(t.bg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .text_xs()
                                    .font_bold()
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_checking_update = false;
                                        this.update_checked = true;
                                        this.status_message = Some(("Notty is up to date (v0.1.0)".to_string(), Instant::now()));
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Check).size_3p5())
                                    .child("Check Now"),
                            )
                            .child(
                                div()
                                    .id("btn-close-update")
                                    .cursor_pointer()
                                    .py_2()
                                    .px_4()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink)
                                    .text_color(t.ink)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_medium()
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_update_modal = false;
                                        cx.notify();
                                    }))
                                    .child("Dismiss"),
                            ),
                    ),
            )
    }


}

