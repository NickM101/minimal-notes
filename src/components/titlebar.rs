use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    IconName, Sizable, TitleBar,
};
use gpui_kit::*;
use std::path::PathBuf;
use crate::theme::AppTheme;
use crate::NotesApp;

impl NotesApp {
    pub(crate) fn render_titlebar(&self, t: &AppTheme, cx: &mut Context<Self>) -> impl IntoElement {
        let logo_path = self.logo_png_path.clone();

        TitleBar::new()
            .bg(t.bg)
            .border_color(t.ink_faint)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(render_titlebar_brand(&logo_path, t))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .pr_2()
                            .child(
                                Button::new("btn-titlebar-settings")
                                    .icon(IconName::Settings)
                                    .ghost()
                                    .small()
                                    .tooltip("Settings")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_settings_modal = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-titlebar-update")
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
                            ),
                    ),
            )
    }
}

pub fn render_titlebar_brand(logo_png_path: &Option<PathBuf>, t: &AppTheme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .child(if let Some(path) = logo_png_path {
            div()
                .w(px(16.0))
                .h(px(16.0))
                .rounded_sm()
                .overflow_hidden()
                .child(
                    img(path.clone())
                        .w(px(16.0))
                        .h(px(16.0)),
                )
                .into_any_element()
        } else {
            div()
                .w(px(14.0))
                .h(px(14.0))
                .rounded_sm()
                .bg(t.ink)
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(t.bg)
                .child("N")
                .into_any_element()
        })
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(t.ink)
                .child("Notty"),
        )
}
