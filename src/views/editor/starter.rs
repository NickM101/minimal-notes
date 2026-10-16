use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::base::StyledExt;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    IconName, Sizable,
};
use gpui_kit::*;

impl NotesApp {
    pub(crate) fn render_notion_empty_starter(
        &self,
        t: &AppTheme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .py_12()
            .px_6()
            .border_1()
            .border_dashed()
            .border_color(t.ink_faint)
            .rounded_lg()
            .bg(t.surface)
            .child(
                div()
                    .text_base()
                    .font_bold()
                    .text_color(t.ink)
                    .mb_1()
                    .child("Notty — Live Visual Canvas"),
            )
            .child(div().text_xs().text_color(t.ink_muted).mb_5().child(
                "Type freely, switch to Source mode, or pick a Notion-style block to begin:",
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_center()
                    .gap_2p5()
                    .child(
                        Button::new("starter-checklist")
                            .icon(IconName::Check)
                            .ghost()
                            .small()
                            .label("+ Checklist Task")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("checklist", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-heading")
                            .ghost()
                            .small()
                            .label("+ Heading")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("h1", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-code")
                            .icon(IconName::Copy)
                            .ghost()
                            .small()
                            .label("+ Code Snippet")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("code", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-quote")
                            .ghost()
                            .small()
                            .label("+ Quote")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("quote", window, cx);
                            })),
                    )
                    .child(
                        Button::new("starter-callout")
                            .icon(IconName::Info)
                            .ghost()
                            .small()
                            .label("+ Callout Note")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_formatting("callout", window, cx);
                            })),
                    ),
            )
    }
}
