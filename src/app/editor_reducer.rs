//! Pure document reducer for the block editor, ported from cascade-editor's
//! unidirectional core: `dispatch(action)` → `reduce(blocks, action)` → effects.
//!
//! This module has **zero GPUI imports** so the document semantics can be unit
//! tested without a window. `NotesApp::dispatch` in `src/app/blocks.rs` applies
//! the returned effects (focus, save) to the live input states.

use crate::models::{Block, BlockKind};

/// Every document mutation funnels through one of these.
#[derive(Debug, Clone)]
pub enum EditorAction {
    /// Split the block at `split_pos` (a byte offset into its text); the tail
    /// becomes a new block inserted after it.
    SplitBlock {
        id: String,
        split_pos: usize,
    },
    /// Backspace at the start of a block: non-paragraph blocks convert to
    /// paragraph in place; paragraphs merge into the previous block.
    MergeWithPrevious {
        id: String,
    },
    /// Replace the block's kind wholesale (kind carries the FINAL text).
    ConvertBlock {
        id: String,
        kind: BlockKind,
    },
    DeleteBlock {
        id: String,
    },
    DuplicateBlock {
        id: String,
    },
    /// Indent (+1) / outdent (-1); clamped 0..=5, Bullet/Numbered only.
    Indent {
        id: String,
        delta: i8,
    },
    /// Insert a new block; `None` appends at the end.
    InsertBlock {
        after_idx: Option<usize>,
        kind: BlockKind,
    },
    /// Directly set a block's text (used to strip "/query" slash triggers).
    SetText {
        id: String,
        text: String,
    },
}

/// Side effects the reducer cannot perform itself; applied by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorEffect {
    /// Focus a block's input; `None` cursor means end of text.
    FocusBlock {
        id: String,
        cursor: Option<usize>,
    },
    /// Reserved for actions that need to move focus to the title
    /// (e.g. backspace-merge from the very first block).
    #[allow(dead_code)]
    FocusTitle,
    RequestSave,
}

/// Applies `action` to `blocks` in place and returns the effects to apply.
pub fn reduce(blocks: &mut Vec<Block>, action: &EditorAction) -> Vec<EditorEffect> {
    match action {
        EditorAction::SplitBlock { id, split_pos } => split_block(blocks, id, *split_pos),
        EditorAction::MergeWithPrevious { id } => merge_with_previous(blocks, id),
        EditorAction::ConvertBlock { id, kind } => convert_block(blocks, id, kind.clone()),
        EditorAction::DeleteBlock { id } => delete_block(blocks, id),
        EditorAction::DuplicateBlock { id } => duplicate_block(blocks, id),
        EditorAction::Indent { id, delta } => indent_block(blocks, id, *delta),
        EditorAction::InsertBlock { after_idx, kind } => {
            insert_block(blocks, *after_idx, kind.clone())
        }
        EditorAction::SetText { id, text } => set_text(blocks, id, text.clone()),
    }
}

fn split_block(blocks: &mut Vec<Block>, id: &str, split_pos: usize) -> Vec<EditorEffect> {
    let idx = match blocks.iter().position(|b| b.id == id) {
        Some(i) => i,
        None => return vec![],
    };

    let text = blocks[idx].text().to_string();
    // Clamp to char boundaries so a cursor landing mid-codepoint can't panic.
    let split_pos = clamp_to_char_boundary(&text, split_pos);
    let head = text[..split_pos].to_string();
    let tail = text[split_pos..].to_string();

    // The tail inherits the list semantics of the block it was split from,
    // matching the long-standing behavior of the editor.
    let tail_kind = match &blocks[idx].kind {
        BlockKind::Bullet { indent, .. } => BlockKind::Bullet {
            text: tail,
            indent: *indent,
        },
        BlockKind::Numbered { indent, .. } => BlockKind::Numbered {
            text: tail,
            indent: *indent,
        },
        // Splitting off a to-do always starts a fresh unchecked one
        BlockKind::Todo { .. } => BlockKind::Todo {
            text: tail,
            checked: false,
        },
        _ => BlockKind::Paragraph { text: tail },
    };

    blocks[idx].set_text(head);
    let new_block = Block::new(tail_kind);
    let new_id = new_block.id.clone();
    blocks.insert(idx + 1, new_block);

    vec![
        EditorEffect::FocusBlock {
            id: new_id,
            cursor: Some(0),
        },
        EditorEffect::RequestSave,
    ]
}

fn merge_with_previous(blocks: &mut Vec<Block>, id: &str) -> Vec<EditorEffect> {
    let idx = match blocks.iter().position(|b| b.id == id) {
        Some(i) => i,
        None => return vec![],
    };

    // Non-paragraph blocks first collapse to a paragraph instead of merging,
    // so backspace progressively "unwraps" formatting. Matches current behavior.
    if !matches!(blocks[idx].kind, BlockKind::Paragraph { .. }) {
        let text = blocks[idx].text().to_string();
        blocks[idx].kind = BlockKind::Paragraph { text };
        return vec![
            EditorEffect::FocusBlock {
                id: id.to_string(),
                cursor: Some(0),
            },
            EditorEffect::RequestSave,
        ];
    }

    if idx == 0 {
        return vec![];
    }

    let prev_len = blocks[idx - 1].text().len();
    let curr_text = blocks[idx].text().to_string();
    let prev_id = blocks[idx - 1].id.clone();
    let prev_text = blocks[idx - 1].text().to_string();
    blocks[idx - 1].set_text(format!("{}{}", prev_text, curr_text));
    blocks.remove(idx);

    vec![
        EditorEffect::FocusBlock {
            id: prev_id,
            cursor: Some(prev_len),
        },
        EditorEffect::RequestSave,
    ]
}

fn convert_block(blocks: &mut Vec<Block>, id: &str, kind: BlockKind) -> Vec<EditorEffect> {
    match blocks.iter_mut().find(|b| b.id == id) {
        Some(blk) => blk.kind = kind,
        None => return vec![],
    }
    vec![EditorEffect::RequestSave]
}

fn delete_block(blocks: &mut Vec<Block>, id: &str) -> Vec<EditorEffect> {
    if blocks.len() > 1 {
        blocks.retain(|b| b.id != id);
    } else if let Some(first) = blocks.first_mut() {
        // Never leave the document empty: reset the last block instead.
        first.kind = BlockKind::Paragraph {
            text: String::new(),
        };
    }
    vec![EditorEffect::RequestSave]
}

fn duplicate_block(blocks: &mut Vec<Block>, id: &str) -> Vec<EditorEffect> {
    let idx = match blocks.iter().position(|b| b.id == id) {
        Some(i) => i,
        None => return vec![],
    };
    let copy = Block::new(blocks[idx].kind.clone());
    blocks.insert(idx + 1, copy);
    vec![EditorEffect::RequestSave]
}

fn indent_block(blocks: &mut Vec<Block>, id: &str, delta: i8) -> Vec<EditorEffect> {
    if let Some(blk) = blocks.iter_mut().find(|b| b.id == id) {
        match &mut blk.kind {
            BlockKind::Bullet { indent, .. } | BlockKind::Numbered { indent, .. } => {
                let next = *indent as i8 + delta;
                *indent = next.clamp(0, 5) as u8;
                return vec![EditorEffect::RequestSave];
            }
            _ => {}
        }
    }
    vec![]
}

fn insert_block(
    blocks: &mut Vec<Block>,
    after_idx: Option<usize>,
    kind: BlockKind,
) -> Vec<EditorEffect> {
    let new_block = Block::new(kind);
    let new_id = new_block.id.clone();
    match after_idx {
        Some(i) if i < blocks.len() => blocks.insert(i + 1, new_block),
        _ => blocks.push(new_block),
    }
    vec![
        EditorEffect::FocusBlock {
            id: new_id,
            cursor: Some(0),
        },
        EditorEffect::RequestSave,
    ]
}

fn set_text(blocks: &mut Vec<Block>, id: &str, text: String) -> Vec<EditorEffect> {
    match blocks.iter_mut().find(|b| b.id == id) {
        Some(blk) => blk.set_text(text),
        None => return vec![],
    }
    vec![EditorEffect::RequestSave]
}

fn clamp_to_char_boundary(text: &str, mut pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Markdown prefix shortcuts, checked on every `Change`. Order matters:
/// longest prefixes first. Mirrors the original `try_convert_markdown_shortcut`.
pub fn markdown_shortcut(text: &str) -> Option<BlockKind> {
    if let Some(rest) = text.strip_prefix("### ") {
        return Some(BlockKind::Heading {
            level: 3,
            text: rest.to_string(),
        });
    }
    if let Some(rest) = text.strip_prefix("## ") {
        return Some(BlockKind::Heading {
            level: 2,
            text: rest.to_string(),
        });
    }
    if let Some(rest) = text.strip_prefix("# ") {
        return Some(BlockKind::Heading {
            level: 1,
            text: rest.to_string(),
        });
    }
    if let Some(rest) = text.strip_prefix("- [ ] ") {
        return Some(BlockKind::Todo {
            text: rest.to_string(),
            checked: false,
        });
    }
    if let Some(rest) = text.strip_prefix("- [x] ") {
        return Some(BlockKind::Todo {
            text: rest.to_string(),
            checked: true,
        });
    }
    if let Some(rest) = text.strip_prefix("[] ") {
        return Some(BlockKind::Todo {
            text: rest.to_string(),
            checked: false,
        });
    }
    if let Some(rest) = text.strip_prefix("- ") {
        return Some(BlockKind::Bullet {
            text: rest.to_string(),
            indent: 0,
        });
    }
    if let Some(rest) = text.strip_prefix("* ") {
        return Some(BlockKind::Bullet {
            text: rest.to_string(),
            indent: 0,
        });
    }
    if let Some(rest) = text.strip_prefix("1. ") {
        return Some(BlockKind::Numbered {
            text: rest.to_string(),
            indent: 0,
        });
    }
    if let Some(rest) = text.strip_prefix("> ") {
        return Some(BlockKind::Quote {
            text: rest.to_string(),
        });
    }
    if let Some(after_ticks) = text.strip_prefix("```") {
        let lang = after_ticks.trim();
        return Some(BlockKind::Code {
            language: if lang.is_empty() {
                "rust".to_string()
            } else {
                lang.to_string()
            },
            text: String::new(),
        });
    }
    if text == "---" {
        return Some(BlockKind::Divider);
    }
    None
}

/// [`markdown_shortcut`], gated to changes that look like typing one
/// character. A paste can legitimately begin with "# " or "```"; converting
/// it would rewrite or wipe the pasted content, so a multi-character jump
/// (a paste, an IME commit, a programmatic replace) never converts.
pub fn typing_markdown_shortcut(old_text: &str, new_text: &str) -> Option<BlockKind> {
    if new_text.chars().count() > old_text.chars().count() + 1 {
        return None;
    }
    markdown_shortcut(new_text)
}

/// Input placeholder shown for each block kind.
pub fn placeholder_for_kind(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::Paragraph { .. } => "Type '/' for commands...",
        BlockKind::Heading { .. } => "Heading...",
        BlockKind::Bullet { .. } | BlockKind::Numbered { .. } => "List item...",
        BlockKind::Todo { .. } => "To-do...",
        BlockKind::Quote { .. } => "Empty quote...",
        BlockKind::Code { .. } => "Code...",
        _ => "",
    }
}

/// Stable discriminant for a kind, used to detect placeholder changes without
/// touching the input state on every reconcile.
pub fn kind_key(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::Paragraph { .. } => "paragraph",
        BlockKind::Heading { .. } => "heading",
        BlockKind::Bullet { .. } => "bullet",
        BlockKind::Numbered { .. } => "numbered",
        BlockKind::Todo { .. } => "todo",
        BlockKind::Quote { .. } => "quote",
        BlockKind::Code { .. } => "code",
        BlockKind::Divider => "divider",
        BlockKind::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn para(text: &str) -> Block {
        Block::new(BlockKind::Paragraph {
            text: text.to_string(),
        })
    }

    fn blocks(v: &[&str]) -> Vec<Block> {
        v.iter().map(|t| para(t)).collect()
    }

    fn text_of(blocks: &[Block], id: &str) -> String {
        blocks
            .iter()
            .find(|b| b.id == id)
            .unwrap()
            .text()
            .to_string()
    }

    #[test]
    fn split_mid_text_moves_tail_to_new_block() {
        let mut b = blocks(&["hello world"]);
        let id = b[0].id.clone();
        let effects = reduce(
            &mut b,
            &EditorAction::SplitBlock {
                id: id.clone(),
                split_pos: 5,
            },
        );
        assert_eq!(b.len(), 2);
        assert_eq!(text_of(&b, &id), "hello");
        assert_eq!(b[1].text(), " world");
        assert!(matches!(b[1].kind, BlockKind::Paragraph { .. }));
        assert!(effects.contains(&EditorEffect::FocusBlock {
            id: b[1].id.clone(),
            cursor: Some(0)
        }));
        assert!(effects.contains(&EditorEffect::RequestSave));
    }

    #[test]
    fn split_clamps_to_char_boundary() {
        let mut b = blocks(&["héllo"]); // h=0, é=1..3, l=3
        let id = b[0].id.clone();
        reduce(&mut b, &EditorAction::SplitBlock { id, split_pos: 2 }); // mid-é (inside é's bytes)
        assert_eq!(b[0].text(), "h");
        assert_eq!(b[1].text(), "éllo");
    }

    #[test]
    fn split_inherits_list_kind_and_indent() {
        let mut b = vec![Block::new(BlockKind::Bullet {
            text: "item".into(),
            indent: 2,
        })];
        let id = b[0].id.clone();
        reduce(&mut b, &EditorAction::SplitBlock { id, split_pos: 4 });
        assert!(matches!(&b[1].kind, BlockKind::Bullet { indent: 2, .. }));
        let mut b = vec![Block::new(BlockKind::Todo {
            text: "task".into(),
            checked: true,
        })];
        let id = b[0].id.clone();
        reduce(&mut b, &EditorAction::SplitBlock { id, split_pos: 4 });
        assert!(matches!(&b[1].kind, BlockKind::Todo { checked: false, .. }));
    }

    #[test]
    fn merge_concatenates_and_focuses_junction() {
        let mut b = blocks(&["abc", "def"]);
        let second = b[1].id.clone();
        let effects = reduce(
            &mut b,
            &EditorAction::MergeWithPrevious { id: second.clone() },
        );
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].text(), "abcdef");
        assert!(effects.contains(&EditorEffect::FocusBlock {
            id: b[0].id.clone(),
            cursor: Some(3)
        }));
    }

    #[test]
    fn merge_first_block_is_noop() {
        let mut b = blocks(&["only"]);
        let id = b[0].id.clone();
        let effects = reduce(&mut b, &EditorAction::MergeWithPrevious { id });
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].text(), "only");
        assert!(effects.is_empty());
    }

    #[test]
    fn merge_non_paragraph_converts_instead() {
        let mut b = vec![
            para("prev"),
            Block::new(BlockKind::Heading {
                level: 1,
                text: "title".into(),
            }),
        ];
        let id = b[1].id.clone();
        let effects = reduce(&mut b, &EditorAction::MergeWithPrevious { id: id.clone() });
        assert_eq!(b.len(), 2);
        assert!(matches!(&b[1].kind, BlockKind::Paragraph { text } if text == "title"));
        assert!(effects.contains(&EditorEffect::FocusBlock {
            id,
            cursor: Some(0)
        }));
    }

    #[test]
    fn convert_keeps_id_and_final_text() {
        let mut b = blocks(&["/todo buy milk"]);
        let id = b[0].id.clone();
        reduce(
            &mut b,
            &EditorAction::ConvertBlock {
                id,
                kind: BlockKind::Todo {
                    text: "buy milk".into(),
                    checked: false,
                },
            },
        );
        assert!(
            matches!(&b[0].kind, BlockKind::Todo { text, checked: false } if text == "buy milk")
        );
    }

    #[test]
    fn delete_removes_but_last_block_resets() {
        let mut b = blocks(&["a", "b"]);
        let second = b[1].id.clone();
        reduce(&mut b, &EditorAction::DeleteBlock { id: second });
        assert_eq!(b.len(), 1);

        let id = b[0].id.clone();
        reduce(&mut b, &EditorAction::DeleteBlock { id });
        assert_eq!(b.len(), 1);
        assert!(matches!(&b[0].kind, BlockKind::Paragraph { text } if text.is_empty()));
    }

    #[test]
    fn duplicate_gets_fresh_id_and_same_kind() {
        let mut b = vec![Block::new(BlockKind::Quote { text: "q".into() })];
        let id = b[0].id.clone();
        reduce(&mut b, &EditorAction::DuplicateBlock { id });
        assert_eq!(b.len(), 2);
        assert_ne!(b[0].id, b[1].id);
        assert_eq!(b[0].kind, b[1].kind);
    }

    #[test]
    fn indent_clamps_and_ignores_paragraphs() {
        let mut b = vec![Block::new(BlockKind::Bullet {
            text: "x".into(),
            indent: 0,
        })];
        let id = b[0].id.clone();
        reduce(
            &mut b,
            &EditorAction::Indent {
                id: id.clone(),
                delta: -1,
            },
        );
        assert!(matches!(&b[0].kind, BlockKind::Bullet { indent: 0, .. }));
        reduce(
            &mut b,
            &EditorAction::Indent {
                id: id.clone(),
                delta: 9,
            },
        );
        assert!(matches!(&b[0].kind, BlockKind::Bullet { indent: 5, .. }));

        let mut p = blocks(&["plain"]);
        let pid = p[0].id.clone();
        let effects = reduce(&mut p, &EditorAction::Indent { id: pid, delta: 1 });
        assert!(effects.is_empty());
        assert!(matches!(&p[0].kind, BlockKind::Paragraph { .. }));
    }

    #[test]
    fn insert_block_appends_or_splices() {
        let mut b = blocks(&["a", "c"]);
        let effects = reduce(
            &mut b,
            &EditorAction::InsertBlock {
                after_idx: Some(0),
                kind: BlockKind::Paragraph { text: "b".into() },
            },
        );
        assert_eq!(b.len(), 3);
        assert_eq!(b[1].text(), "b");
        assert!(effects.contains(&EditorEffect::RequestSave));

        reduce(
            &mut b,
            &EditorAction::InsertBlock {
                after_idx: Some(99),
                kind: BlockKind::Divider,
            },
        );
        assert!(matches!(b.last().unwrap().kind, BlockKind::Divider));
    }

    #[test]
    fn set_text_updates_and_save_requested() {
        let mut b = blocks(&["old"]);
        let id = b[0].id.clone();
        let effects = reduce(
            &mut b,
            &EditorAction::SetText {
                id,
                text: "new".into(),
            },
        );
        assert_eq!(b[0].text(), "new");
        assert_eq!(effects, vec![EditorEffect::RequestSave]);
    }

    #[test]
    fn markdown_shortcut_full_table() {
        assert!(matches!(
            markdown_shortcut("# Hi"),
            Some(BlockKind::Heading { level: 1, .. })
        ));
        assert!(matches!(
            markdown_shortcut("## Hi"),
            Some(BlockKind::Heading { level: 2, .. })
        ));
        assert!(matches!(
            markdown_shortcut("### Hi"),
            Some(BlockKind::Heading { level: 3, .. })
        ));
        assert!(matches!(
            markdown_shortcut("- item"),
            Some(BlockKind::Bullet { .. })
        ));
        assert!(matches!(
            markdown_shortcut("* item"),
            Some(BlockKind::Bullet { .. })
        ));
        assert!(matches!(
            markdown_shortcut("1. item"),
            Some(BlockKind::Numbered { .. })
        ));
        assert!(matches!(
            markdown_shortcut("- [ ] task"),
            Some(BlockKind::Todo { checked: false, .. })
        ));
        assert!(matches!(
            markdown_shortcut("- [x] task"),
            Some(BlockKind::Todo { checked: true, .. })
        ));
        assert!(matches!(
            markdown_shortcut("[] task"),
            Some(BlockKind::Todo { checked: false, .. })
        ));
        assert!(matches!(
            markdown_shortcut("> quote"),
            Some(BlockKind::Quote { .. })
        ));
        assert!(matches!(
            markdown_shortcut("```"),
            Some(BlockKind::Code { .. })
        ));
        assert!(matches!(
            markdown_shortcut("```rs"),
            Some(BlockKind::Code { .. })
        ));
        assert!(matches!(markdown_shortcut("---"), Some(BlockKind::Divider)));
        assert!(markdown_shortcut("plain").is_none());
        assert!(markdown_shortcut("#").is_none()); // needs the space
                                                   // Longest prefix wins: "### " is checked before "# "
        assert!(matches!(
            markdown_shortcut("### x"),
            Some(BlockKind::Heading { level: 3, .. })
        ));
    }

    #[test]
    fn typing_markdown_shortcut_allows_typing_and_rejects_paste_jumps() {
        // Typing one character at a time still converts.
        assert!(matches!(
            typing_markdown_shortcut("# Hi", "# Hi!"),
            Some(BlockKind::Heading { level: 1, .. })
        ));
        assert!(matches!(
            typing_markdown_shortcut("#", "# "),
            Some(BlockKind::Heading { level: 1, .. })
        ));
        // A paste jump (grew by more than one char) never converts, even
        // when the pasted text begins with a trigger prefix.
        assert!(typing_markdown_shortcut("", "```rust\nfn main() {}").is_none());
        assert!(typing_markdown_shortcut("", "# Pasted heading").is_none());
        // Plain typing that matches no shortcut is unchanged.
        assert!(typing_markdown_shortcut("hel", "hello").is_none());
    }

    #[test]
    fn placeholder_and_kind_keys_cover_all_kinds() {
        for kind in [
            BlockKind::Paragraph {
                text: String::new(),
            },
            BlockKind::Heading {
                level: 1,
                text: String::new(),
            },
            BlockKind::Bullet {
                text: String::new(),
                indent: 0,
            },
            BlockKind::Numbered {
                text: String::new(),
                indent: 0,
            },
            BlockKind::Todo {
                text: String::new(),
                checked: false,
            },
            BlockKind::Quote {
                text: String::new(),
            },
            BlockKind::Code {
                text: String::new(),
                language: "rust".into(),
            },
            BlockKind::Divider,
            BlockKind::Unknown,
        ] {
            let _ = placeholder_for_kind(&kind);
            let _ = kind_key(&kind);
        }
        assert_eq!(
            kind_key(&BlockKind::Paragraph { text: "a".into() }),
            "paragraph"
        );
        assert_ne!(
            kind_key(&BlockKind::Paragraph { text: "a".into() }),
            kind_key(&BlockKind::Quote { text: "a".into() }),
        );
    }
}
