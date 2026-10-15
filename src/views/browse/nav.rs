use gpui_kit::assets::IconName as FullIconName;
use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::{button::{Button, ButtonVariants}, Icon, IconName};
use crate::theme::{AppTheme, CurrentScreen};
use crate::{BrowseLayout, NotesApp};

pub fn render_browse_nav(this: &NotesApp, t: &AppTheme, cx: &mut Context<NotesApp>) -> impl IntoElement {
    div()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .px_6()
        .py_3p5()
        .border_b_1()
        .border_color(t.ink_faint)
        .child(
            div()
                .id("btn-browse-back")
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .px_2p5()
                .py_1p5()
                .rounded_md()
                .text_color(t.ink_muted)
                .hover(move |s| s.text_color(t.ink).bg(t.bg_hover))
                .active(move |s| s.bg(t.bg_active))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.current_screen = CurrentScreen::Home;
                    cx.notify();
                }))
                .child(Icon::new(IconName::ChevronLeft).size_4())
                .child(
                    div()
                        .text_sm()
                        .font_medium()
                        .child("Back to Home"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // Search toggle button
                .child(
                    Button::new("btn-browse-search-toggle")
                        .icon(IconName::Search)
                        .ghost()
                        .tooltip(if this.browse_search_open { "Hide search" } else { "Search notes" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.browse_search_open = !this.browse_search_open;
                            cx.notify();
                        })),
                )
                // Layout toggle: Grid vs List
                .child(
                    Button::new("btn-browse-layout-toggle")
                        .icon(match this.browse_layout {
                            BrowseLayout::List => FullIconName::LayoutGrid,
                            BrowseLayout::Grid => FullIconName::LayoutList,
                        })
                        .ghost()
                        .tooltip(match this.browse_layout {
                            BrowseLayout::List => "Switch to Grid view",
                            BrowseLayout::Grid => "Switch to List view",
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.browse_layout = match this.browse_layout {
                                BrowseLayout::List => BrowseLayout::Grid,
                                BrowseLayout::Grid => BrowseLayout::List,
                            };
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("btn-browse-theme-toggle")
                        .icon(if t.is_dark {
                            IconName::Sun
                        } else {
                            IconName::Moon
                        })
                        .ghost()
                        .tooltip(if t.is_dark { "Light mode" } else { "Dark mode" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_theme(cx);
                        })),
                )
                .child(this.render_avatar_button(t, cx)),
        )
}
