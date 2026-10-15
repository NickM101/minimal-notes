use gpui_kit::base::StyledExt;
use gpui_kit::*;
use crate::theme::{AppTheme, AVATAR_ICON_SVG};
use crate::NotesApp;

impl NotesApp {
    pub(crate) fn render_avatar_button(&self, t: &AppTheme, cx: &mut Context<Self>) -> AnyElement {
        let is_signed = self.is_signed_in;
        let user_name = self.user_name.clone();

        avatar_button(
            &user_name,
            is_signed,
            t,
            cx.listener(|this, _, _, cx| {
                this.show_auth_modal = true;
                cx.notify();
            }),
        )
        .into_any_element()
    }
}

pub fn avatar_button(
    user_name: &str,
    is_signed: bool,
    t: &AppTheme,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let initials = if is_signed {
        let parts: Vec<&str> = user_name.split_whitespace().collect();
        if parts.len() >= 2 {
            format!(
                "{}{}",
                parts[0].chars().next().unwrap_or('N'),
                parts[1].chars().next().unwrap_or('M')
            )
        } else {
            user_name.chars().take(2).collect::<String>().to_uppercase()
        }
    } else {
        "Sign In".to_string()
    };

    div()
        .id("btn-avatar-auth")
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(if is_signed { t.ink } else { t.ink_faint })
        .bg(if is_signed { t.surface } else { t.bg })
        .hover(move |s| s.bg(t.bg_hover).border_color(t.ink))
        .active(move |s| s.bg(t.bg_active))
        .on_click(on_click)
        // Avatar circle badge
        .child(
            div()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .bg(if is_signed { t.ink } else { t.surface })
                .flex()
                .items_center()
                .justify_center()
                .text_color(if is_signed { t.bg } else { t.ink })
                .text_xs()
                .font_bold()
                .relative()
                .child(if is_signed {
                    div().child(initials)
                } else {
                    div()
                        .w(px(14.0))
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .data(AVATAR_ICON_SVG.as_bytes())
                                .w(px(13.0))
                                .h(px(13.0))
                                .text_color(t.ink),
                        )
                })
                // Online / Synced status dot
                .child(
                    div()
                        .absolute()
                        .right(px(-1.0))
                        .bottom(px(-1.0))
                        .w(px(7.0))
                        .h(px(7.0))
                        .rounded_full()
                        .border_1()
                        .border_color(t.bg)
                        .bg(if is_signed {
                            hsla(140.0 / 360.0, 0.80, 0.45, 1.0)
                        } else {
                            t.ink_muted
                        }),
                ),
        )
        // Label next to avatar
        .child(
            div()
                .text_xs()
                .font_medium()
                .text_color(t.ink)
                .pr_1()
                .child(if is_signed {
                    user_name.to_string()
                } else {
                    "Sign In".to_string()
                }),
        )
}
