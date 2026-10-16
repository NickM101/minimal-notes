use crate::models::BlockKind;
use crate::theme::AppTheme;
use crate::NotesApp;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(crate) type SlashCommand = (
    &'static str,
    &'static str,
    &'static str,
    Option<IconName>,
    &'static str,
);

/// The single source of truth for slash-menu commands: (id, title, description, icon, badge).
/// Both the menu rendering and the keyboard navigation in `app/blocks.rs` derive from this,
/// so the highlighted row and the executed command can never disagree.
pub(crate) const SLASH_COMMANDS: &[SlashCommand] = &[
    ("text", "Text", "Plain text block", None, "T"),
    ("h1", "Heading 1", "Large section heading", None, "H1"),
    ("h2", "Heading 2", "Medium section heading", None, "H2"),
    ("h3", "Heading 3", "Small section heading", None, "H3"),
    (
        "bullet",
        "Bulleted list",
        "Simple bulleted list",
        Some(IconName::List),
        "•",
    ),
    (
        "number",
        "Numbered list",
        "List with ordered numbers",
        None,
        "1.",
    ),
    (
        "todo",
        "To-do list",
        "Track tasks with a checkbox",
        Some(IconName::ListTodo),
        "[]",
    ),
    ("quote", "Quote", "Capture a quote or callout", None, "\""),
    (
        "code",
        "Code",
        "Capture a code snippet",
        Some(IconName::SquareCode),
        "</>",
    ),
    (
        "divider",
        "Divider",
        "Visually divide blocks with a line",
        None,
        "—",
    ),
    (
        "duplicate",
        "Duplicate",
        "Make a copy of this block",
        Some(IconName::Copy),
        "+",
    ),
    (
        "delete",
        "Delete",
        "Delete this block",
        Some(IconName::Delete),
        "×",
    ),
];

pub(crate) fn filtered_slash_command_ids(query: &str) -> Vec<&'static str> {
    let filter = query.to_lowercase();
    SLASH_COMMANDS
        .iter()
        .filter(|(id, title, desc, _, _)| {
            filter.is_empty()
                || id.contains(&filter)
                || title.to_lowercase().contains(&filter)
                || desc.to_lowercase().contains(&filter)
        })
        .map(|(id, _, _, _, _)| *id)
        .collect()
}

/// Applies a slash command to `block_id`, first stripping the typed "/query"
/// trigger text so it doesn't leak into the converted block's content.
pub(crate) fn execute_slash_command(
    this: &mut NotesApp,
    cmd: &str,
    block_id: &str,
    window: &mut Window,
    cx: &mut Context<NotesApp>,
) {
    let query = this
        .slash_menu_target
        .as_ref()
        .filter(|(id, _)| id == block_id)
        .map(|(_, q)| q.clone())
        .unwrap_or_default();

    if !query.is_empty() {
        let trigger = format!("/{}", query);
        if let Some(id) = this.selected_id.clone() {
            if let Some(note) = this.notes.iter_mut().find(|n| n.id == id) {
                if let Some(blk) = note.blocks.iter_mut().find(|b| b.id == block_id) {
                    let text = blk.text().to_string();
                    if let Some(stripped) = text.strip_prefix(&trigger) {
                        blk.set_text(stripped.to_string());
                        note.updated_at = chrono::Local::now();
                        let _ = this.storage.save(note);
                    }
                }
            }
        }
    }

    this.slash_menu_target = None;

    use crate::app::editor_reducer::EditorAction;
    match cmd {
        "text" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Paragraph {
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "h1" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Heading {
                    level: 1,
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "h2" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Heading {
                    level: 2,
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "h3" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Heading {
                    level: 3,
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "bullet" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Bullet {
                    text: current_text(this, block_id),
                    indent: 0,
                },
            },
            window,
            cx,
        ),
        "number" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Numbered {
                    text: current_text(this, block_id),
                    indent: 0,
                },
            },
            window,
            cx,
        ),
        "todo" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Todo {
                    text: current_text(this, block_id),
                    checked: false,
                },
            },
            window,
            cx,
        ),
        "quote" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Quote {
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "code" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Code {
                    language: "rust".to_string(),
                    text: current_text(this, block_id),
                },
            },
            window,
            cx,
        ),
        "divider" => this.dispatch(
            EditorAction::ConvertBlock {
                id: block_id.to_string(),
                kind: BlockKind::Divider,
            },
            window,
            cx,
        ),
        "duplicate" => this.dispatch(
            EditorAction::DuplicateBlock {
                id: block_id.to_string(),
            },
            window,
            cx,
        ),
        "delete" => this.dispatch(
            EditorAction::DeleteBlock {
                id: block_id.to_string(),
            },
            window,
            cx,
        ),
        _ => {}
    }
}

/// Reads a block's current model text (post-strip) for the final kind.
fn current_text(this: &NotesApp, block_id: &str) -> String {
    this.selected_note()
        .and_then(|n| n.blocks.iter().find(|b| b.id == block_id))
        .map(|b| b.text().to_string())
        .unwrap_or_default()
}

pub fn render_slash_menu(
    block_id: &str,
    filter: &str,
    selection: Option<usize>,
    t: &AppTheme,
    cx: &mut Context<NotesApp>,
) -> impl IntoElement {
    let filtered_cmds: Vec<SlashCommand> = SLASH_COMMANDS
        .iter()
        .copied()
        .filter(|(id, title, desc, _, _)| {
            let f = filter.to_lowercase();
            f.is_empty()
                || id.contains(&f)
                || title.to_lowercase().contains(&f)
                || desc.to_lowercase().contains(&f)
        })
        .collect();

    let bid = block_id.to_string();
    let selected_idx = selection.unwrap_or(0);

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
        .children(filtered_cmds.into_iter().enumerate().map(
            |(item_idx, (cmd_id, title, desc, maybe_icon, badge))| {
                let target_bid = bid.clone();
                let is_selected = item_idx == selected_idx;

                div()
                    .id(SharedString::from(format!("slash-cmd-{}", cmd_id)))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .px_2()
                    .py_1p5()
                    .rounded_sm()
                    .when(is_selected, |s| {
                        s.bg(hsla(0.0, 0.0, 0.5, 0.12))
                            .border_l(px(2.0))
                            .border_color(t.highlight)
                    })
                    .when(!is_selected, |s| {
                        s.hover(|h| h.bg(hsla(0.0, 0.0, 0.5, 0.08)))
                    })
                    .child(
                        div()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded_sm()
                            .bg(t.bg)
                            .border_1()
                            .border_color(if is_selected {
                                t.highlight
                            } else {
                                t.ink_faint
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if let Some(icon) = maybe_icon {
                                gpui_kit::component::Icon::new(icon)
                                    .size_3p5()
                                    .text_color(t.ink)
                                    .into_any_element()
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
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(t.ink)
                                    .child(title),
                            )
                            .child(div().text_xs().text_color(t.ink_muted).child(desc)),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        execute_slash_command(this, cmd_id, &target_bid, window, cx);
                        this.slash_menu_selection = None;
                    }))
            },
        ))
}
