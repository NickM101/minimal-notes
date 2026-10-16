use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::Input,
    Icon, IconName, Sizable,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub fn render_search_bar(
    this: &NotesApp,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .px_6()
        .py_2p5()
        .border_b_1()
        .border_color(t.ink_faint)
        .bg(t.surface)
        .child(
            // Search box with rounded border and subtle background
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(t.ink_faint)
                .bg(t.bg)
                .child(Icon::new(IconName::Search).size_4().text_color(t.ink_muted))
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(&this.search_state).bordered(false).small()),
                )
                .when(!this.search_query.is_empty(), |el| {
                    el.child(
                        Button::new("btn-clear-search")
                            .icon(IconName::Close)
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.search_query.clear();
                                this.search_state.update(cx, |s, cx| {
                                    s.set_value("", window, cx);
                                });
                                cx.notify();
                            })),
                    )
                }),
        )
        .child(
            Button::new("btn-close-search-bar")
                .icon(IconName::Close)
                .ghost()
                .small()
                .tooltip("Close search")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.browse_search_open = false;
                    this.search_query.clear();
                    this.search_state.update(cx, |s, cx| {
                        s.set_value("", window, cx);
                    });
                    cx.notify();
                })),
        )
}
