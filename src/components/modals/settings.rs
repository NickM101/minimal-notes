use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    Icon, IconName, Sizable,
};
use gpui_kit::*;
use std::time::Instant;

impl NotesApp {
    pub(crate) fn render_settings_modal(
        &self,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let auto_save = self.auto_save_interval_sec;
        let font_size_idx = self.font_size_idx;
        let is_dark = self.is_dark_mode;
        let font_size_labels = ["Compact (13px)", "Balanced (15px)", "Relaxed (18px)"];
        let current_font_label = font_size_labels
            .get(font_size_idx)
            .unwrap_or(&"Balanced (15px)");

        div()
            .id("settings-modal-backdrop")
            .absolute()
            .inset_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.75))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_settings_modal = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("settings-modal-card")
                    .w(px(420.0))
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
                                    .child(Icon::new(IconName::Settings).size_5().text_color(t.ink))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child("App Settings"),
                                    ),
                            )
                            .child(
                                div()
                                    .id("btn-close-settings-modal")
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded_sm()
                                    .text_color(t.ink_muted)
                                    .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_settings_modal = false;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::Close).size_4()),
                            ),
                    )
                    // Settings items
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            // Option 1: Theme mode
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_bold()
                                                    .text_color(t.ink)
                                                    .child("Appearance"),
                                            )
                                            .child(div().text_xs().text_color(t.ink_muted).child(
                                                if is_dark {
                                                    "Dark Minimal (#000)"
                                                } else {
                                                    "Old Paper Warmth"
                                                },
                                            )),
                                    )
                                    .child(
                                        Button::new("btn-settings-toggle-theme")
                                            .icon(if is_dark {
                                                IconName::Sun
                                            } else {
                                                IconName::Moon
                                            })
                                            .ghost()
                                            .small()
                                            .label(if is_dark { "Light" } else { "Dark" })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.toggle_theme(cx);
                                            })),
                                    ),
                            )
                            // Option 2: Auto-save interval
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_bold()
                                                    .text_color(t.ink)
                                                    .child("Auto-Save Delay"),
                                            )
                                            .child(div().text_xs().text_color(t.ink_muted).child(
                                                format!("Debounce interval: {}s", auto_save),
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                Button::new("btn-save-delay-1")
                                                    .ghost()
                                                    .small()
                                                    .label("1s")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.auto_save_interval_sec = 1;
                                                        this.status_message = Some((
                                                            "Saved 1s delay".to_string(),
                                                            Instant::now(),
                                                        ));
                                                        cx.notify();
                                                    })),
                                            )
                                            .child(
                                                Button::new("btn-save-delay-2")
                                                    .ghost()
                                                    .small()
                                                    .label("2s")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.auto_save_interval_sec = 2;
                                                        this.status_message = Some((
                                                            "Saved 2s delay".to_string(),
                                                            Instant::now(),
                                                        ));
                                                        cx.notify();
                                                    })),
                                            )
                                            .child(
                                                Button::new("btn-save-delay-5")
                                                    .ghost()
                                                    .small()
                                                    .label("5s")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.auto_save_interval_sec = 5;
                                                        this.status_message = Some((
                                                            "Saved 5s delay".to_string(),
                                                            Instant::now(),
                                                        ));
                                                        cx.notify();
                                                    })),
                                            ),
                                    ),
                            )
                            // Option 3: Editor Typography Scale
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.surface)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_bold()
                                                    .text_color(t.ink)
                                                    .child("Typography Scale"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(t.ink_muted)
                                                    .child(*current_font_label),
                                            ),
                                    )
                                    .child(
                                        Button::new("btn-cycle-font-size")
                                            .ghost()
                                            .small()
                                            .label("Cycle Size")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.font_size_idx = (this.font_size_idx + 1) % 3;
                                                this.status_message = Some((
                                                    "Typography adjusted".to_string(),
                                                    Instant::now(),
                                                ));
                                                cx.notify();
                                            })),
                                    ),
                            )
                            // Storage & Location info
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .p_3()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(t.ink_faint)
                                    .bg(t.bg)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_bold()
                                            .text_color(t.ink)
                                            .child("Storage Location"),
                                    )
                                    .child(div().text_xs().text_color(t.ink_muted).child(
                                        "Local Disk: ./notes_data/*.json (encrypted/persistent)",
                                    )),
                            ),
                    )
                    // Footer Action
                    .child(
                        div()
                            .id("btn-settings-done")
                            .cursor_pointer()
                            .py_2()
                            .rounded_md()
                            .bg(t.ink)
                            .text_color(t.bg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .font_bold()
                            .hover(move |s| s.opacity(0.9))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.show_settings_modal = false;
                                cx.notify();
                            }))
                            .child("Done"),
                    ),
            )
    }
}
