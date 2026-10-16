use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::*;

impl NotesApp {
    pub(crate) fn render_auth_modal(
        &self,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_signed = self.is_signed_in;
        let user_name = self.user_name.clone();
        let user_email = self.user_email.clone();

        div()
            .id("auth-modal-backdrop")
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.75))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_auth_modal = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("auth-modal-card")
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
                    .gap_4()
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                    })
                    // Modal Header
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
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .h(px(24.0))
                                            .rounded_sm()
                                            .bg(t.ink)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_xs()
                                            .font_bold()
                                            .text_color(t.bg)
                                            .child("N"),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child(if is_signed {
                                                "Account Profile"
                                            } else {
                                                "Sign in to Notty"
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-auth-modal")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded_sm()
                                    .text_color(t.ink_muted)
                                    .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Close).size_4()),
                            ),
                    )
                    // Modal Body
                    .child(if is_signed {
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_4()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        div()
                                            .w(px(46.0))
                                            .h(px(46.0))
                                            .rounded_full()
                                            .bg(t.ink)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.bg)
                                            .child("NM"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_bold()
                                                    .text_color(t.ink)
                                                    .child(user_name),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(t.ink_muted)
                                                    .child(user_email),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(hsla(140.0 / 360.0, 0.70, 0.45, 1.0))
                                    .child(
                                        div()
                                            .w(px(8.0))
                                            .h(px(8.0))
                                            .rounded_full()
                                            .bg(hsla(140.0 / 360.0, 0.80, 0.45, 1.0)),
                                    )
                                    .child("Cloud Sync: Active & Synced"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(t.ink_muted)
                                    .child("All your notes, checklists, and voice memos are backed up to your Notty account."),
                            )
                            .child(
                                div()
                                    .id("btn-auth-sign-out")
                                    .cursor_pointer()
                                    .py_2()
                                    .border_1()
                                    .border_color(t.ink)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(t.ink)
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = false;
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child("Sign Out"),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .text_xs()
                                    .line_height(relative(1.5))
                                    .text_color(t.ink_muted)
                                    .child("Sign in to automatically sync your notes across devices and unlock encrypted cloud backup."),
                            )
                            .child(
                                div()
                                    .id("btn-auth-sign-in-nick")
                                    .cursor_pointer()
                                    .p_3()
                                    .rounded_md()
                                    .bg(t.ink)
                                    .text_color(t.bg)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .hover(move |s| s.opacity(0.9))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = true;
                                        this.user_name = "Nick M.".to_string();
                                        this.user_email = "nick@notty.app".to_string();
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_3()
                                            .child(
                                                div()
                                                    .w(px(30.0))
                                                    .h(px(30.0))
                                                    .rounded_full()
                                                    .bg(t.bg)
                                                    .text_color(t.ink)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("NM"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("Sign in as Nick M."),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .opacity(0.7)
                                                            .child("nick@notty.app"),
                                                    ),
                                            ),
                                    )
                                    .child(Icon::new(IconName::ChevronRight).size_4().text_color(t.bg)),
                            )
                            .child(
                                div()
                                    .id("btn-auth-quick-guest")
                                    .cursor_pointer()
                                    .py_2()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .rounded_md()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(t.ink)
                                    .hover(move |s| s.bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.is_signed_in = true;
                                        this.user_name = "Zen Writer".to_string();
                                        this.user_email = "writer@notty.app".to_string();
                                        this.show_auth_modal = false;
                                        cx.notify();
                                    }))
                                    .child("Sign in as Zen Writer"),
                            )
                            .into_any_element()
                    }),
            )
    }
}
