use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::attachment::{ChecklistItem, VoiceRecording};
use crate::models::block::{Block, BlockKind};

/// The Note model supporting schema versioning, backward compatibility with
/// legacy JSON notes (`body` string + `checklist`), and round-tripping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
    // Kept for backward compatibility during deserialization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub voice: Option<VoiceRecording>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

fn default_schema_version() -> u32 {
    1
}

impl Note {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        let now = Local::now();
        let body_str: String = body.into();
        let blocks = if body_str.trim().is_empty() {
            vec![Block::new(BlockKind::Paragraph {
                text: String::new(),
            })]
        } else {
            Self::markdown_to_blocks(&body_str)
        };

        Self {
            id: Uuid::new_v4().to_string(),
            schema_version: 1,
            title: title.into(),
            blocks,
            body: None,
            checklist: Vec::new(),
            images: Vec::new(),
            voice: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Migrates legacy fields (`body` string and `checklist` vector) into `blocks` if `blocks` is empty.
    pub fn migrate_if_needed(&mut self) {
        if self.blocks.is_empty() {
            let mut migrated = Vec::new();
            if let Some(body_text) = &self.body {
                if !body_text.trim().is_empty() {
                    migrated.extend(Self::markdown_to_blocks(body_text));
                }
            }

            // Append checklist items as todo blocks
            for item in &self.checklist {
                migrated.push(Block::with_id(
                    item.id.clone(),
                    BlockKind::Todo {
                        text: item.text.clone(),
                        checked: item.checked,
                    },
                ));
            }

            if migrated.is_empty() {
                migrated.push(Block::new(BlockKind::Paragraph {
                    text: String::new(),
                }));
            }

            self.blocks = migrated;
        }

        self.schema_version = 1;
        self.body = None;
        self.checklist.clear();
    }

    /// Converts markdown string to structured blocks using `crate::markdown::parse_markdown_blocks`.
    pub fn markdown_to_blocks(content: &str) -> Vec<Block> {
        use crate::markdown::{parse_markdown_blocks, MarkdownBlock};
        let parsed = parse_markdown_blocks(content);
        let mut blocks = Vec::new();

        for mb in parsed {
            match mb {
                MarkdownBlock::Blank => {
                    // Blank lines in legacy content can be empty paragraphs
                    blocks.push(Block::new(BlockKind::Paragraph {
                        text: String::new(),
                    }));
                }
                MarkdownBlock::Heading(lvl, text) => {
                    blocks.push(Block::new(BlockKind::Heading {
                        level: (lvl as u8).clamp(1, 3),
                        text,
                    }));
                }
                MarkdownBlock::Checklist {
                    is_checked, text, ..
                } => {
                    blocks.push(Block::new(BlockKind::Todo {
                        text,
                        checked: is_checked,
                    }));
                }
                MarkdownBlock::Bullet(text) => {
                    blocks.push(Block::new(BlockKind::Bullet { text, indent: 0 }));
                }
                MarkdownBlock::Numbered(_, text) => {
                    blocks.push(Block::new(BlockKind::Numbered { text, indent: 0 }));
                }
                MarkdownBlock::Quote(text) => {
                    blocks.push(Block::new(BlockKind::Quote { text }));
                }
                MarkdownBlock::Callout { text, .. } => {
                    // Convert callout to quote block retaining literal markdown callout format
                    blocks.push(Block::new(BlockKind::Quote { text }));
                }
                MarkdownBlock::CodeBlock { language, code } => {
                    blocks.push(Block::new(BlockKind::Code {
                        language,
                        text: code,
                    }));
                }
                MarkdownBlock::Divider => {
                    blocks.push(Block::new(BlockKind::Divider));
                }
                MarkdownBlock::Paragraph(text) => {
                    blocks.push(Block::new(BlockKind::Paragraph { text }));
                }
            }
        }

        if blocks.is_empty() {
            blocks.push(Block::new(BlockKind::Paragraph {
                text: String::new(),
            }));
        }

        blocks
    }

    /// Helper that returns plain text concatenated from all blocks (for search, previews, snippets).
    pub fn plain_text(&self) -> String {
        let texts: Vec<&str> = self
            .blocks
            .iter()
            .map(|b| b.text())
            .filter(|s| !s.is_empty())
            .collect();
        texts.join("\n")
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            "New Note"
        } else {
            self.title.trim()
        }
    }

    pub fn snippet(&self) -> String {
        self.snippet_max(200)
    }

    pub fn snippet_max(&self, max_chars: usize) -> String {
        let text = self.plain_text();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            "No additional text".to_string()
        } else {
            let clean_lines: Vec<&str> = trimmed
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with("```"))
                .map(|l| l.trim_start_matches('#').trim_start_matches('>').trim())
                .filter(|l| !l.is_empty())
                .collect();

            let joined = clean_lines.join(" ");
            if joined.chars().count() > max_chars {
                let s: String = joined.chars().take(max_chars.saturating_sub(3)).collect();
                format!("{}...", s.trim_end())
            } else {
                joined
            }
        }
    }

    pub fn formatted_date(&self) -> String {
        self.updated_at.format("%b %d, %Y").to_string()
    }

    pub fn formatted_full_date(&self) -> String {
        self.updated_at
            .format("%B %-d, %Y at %-I:%M %p")
            .to_string()
    }
}
