use gpui_kit::base::StyledExt;
use gpui_kit::*;
use gpui_kit::component::IconName;
use crate::models::BlockKind;
use crate::theme::AppTheme;
use crate::NotesApp;

pub fn render_slash_menu(
    block_id: &str,
    filter: &str,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    let commands: Vec<(&'static str, &'static str, &'static str, Option<IconName>, &'static str)> = vec![
        ("text", "Text", "Plain text block", None, "T"),
        ("h1", "Heading 1", "Large section heading", None, "H1"),
        ("h2", "Heading 2", "Medium section heading", None, "H2"),
        ("h3", "Heading 3", "Small section heading", None, "H3"),
        ("bullet", "Bulleted list", "Simple bulleted list", Some(IconName::Menu), "•"),
        ("number", "Numbered list", "List with ordered numbers", None, "1."),
        ("todo", "To-do list", "Track tasks with a checkbox", Some(IconName::Check), "[]"),
        ("quote", "Quote", "Capture a quote or callout", None, "\""),
        ("code", "Code", "Capture a code snippet", Some(IconName::Copy), "</>"),
        ("divider", "Divider", "Visually divide blocks with a line", None, "—"),
        ("duplicate", "Duplicate", "Make a copy of this block", Some(IconName::Copy), "+"),
        ("delete", "Delete", "Delete this block", Some(IconName::Delete), "×"),
    ];

    let filtered_cmds: Vec<_> = commands
        .into_iter()
        .filter(|(id, title, desc, _, _)| {
            filter.is_empty()
                || id.contains(filter)
                || title.to_lowercase().contains(filter)
                || desc.to_lowercase().contains(filter)
        })
        .collect();

    let bid = block_id.to_string();

    div()
        .id("slash-command-menu")
        .my_2()
        .w(px(280.0))
        .bg(t.surface)
        .border_1()
        .border_color(t.ink_faint)
        .rounded_md()
        .shadow_md()
        .p_1p5()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            div()
                .px_2()
                .py_1()
                .text_xs()
                .font_bold()
                .text_color(t.ink_muted)
                .child("BASIC BLOCKS"),
        )
        .children(filtered_cmds.into_iter().map(|(cmd_id, title, desc, maybe_icon, badge)| {
            let target_bid = bid.clone();
            div()
                .id(SharedString::from(format!("slash-cmd-{}", cmd_id)))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_2p5()
                .px_2()
                .py_1p5()
                .rounded_sm()
                .hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.12)))
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded_sm()
                        .bg(t.bg)
                        .border_1()
                        .border_color(t.ink_faint)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(if let Some(icon) = maybe_icon {
                            gpui_kit::component::Icon::new(icon).size_3p5().text_color(t.ink).into_any_element()
                        } else {
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(t.ink)
                                .child(badge)
                                .into_any_element()
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_sm().font_semibold().text_color(t.ink).child(title))
                        .child(div().text_xs().text_color(t.ink_muted).child(desc)),
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    match cmd_id {
                        "text" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Paragraph { text: t }, window, cx);
                        }
                        "h1" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Heading { level: 1, text: t }, window, cx);
                        }
                        "h2" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Heading { level: 2, text: t }, window, cx);
                        }
                        "h3" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Heading { level: 3, text: t }, window, cx);
                        }
                        "bullet" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Bullet { text: t, indent: 0 }, window, cx);
                        }
                        "number" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Numbered { text: t, indent: 0 }, window, cx);
                        }
                        "todo" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Todo { text: t, checked: false }, window, cx);
                        }
                        "quote" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Quote { text: t }, window, cx);
                        }
                        "code" => {
                            this.convert_block_type(&target_bid, |t| BlockKind::Code { language: "rust".to_string(), text: t }, window, cx);
                        }
                        "divider" => {
                            this.convert_block_type(&target_bid, |_| BlockKind::Divider, window, cx);
                        }
                        "duplicate" => {
                            this.duplicate_block(&target_bid, window, cx);
                        }
                        "delete" => {
                            this.delete_block(&target_bid, window, cx);
                        }
                        _ => {}
                    }
                }))
        }))
}
