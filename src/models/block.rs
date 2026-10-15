use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Block data model adhering to the target specification:
/// Serde tagged by "type" (snake_case).
/// Includes: paragraph, heading, bullet, numbered, todo, quote, code, divider, unknown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockKind {
    Paragraph {
        text: String,
    },
    Heading {
        level: u8, // 1..=3
        text: String,
    },
    Bullet {
        text: String,
        #[serde(default)]
        indent: u8,
    },
    Numbered {
        text: String,
        #[serde(default)]
        indent: u8,
    },
    Todo {
        text: String,
        checked: bool,
    },
    Quote {
        text: String,
    },
    Code {
        text: String,
        #[serde(default = "default_code_language")]
        language: String,
    },
    Divider,
    #[serde(other)]
    Unknown,
}

fn default_code_language() -> String {
    "rust".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    #[serde(flatten)]
    pub kind: BlockKind,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Block {
    pub fn new(kind: BlockKind) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            extra: HashMap::new(),
        }
    }

    pub fn with_id(id: impl Into<String>, kind: BlockKind) -> Self {
        Self {
            id: id.into(),
            kind,
            extra: HashMap::new(),
        }
    }

    pub fn text(&self) -> &str {
        match &self.kind {
            BlockKind::Paragraph { text }
            | BlockKind::Heading { text, .. }
            | BlockKind::Bullet { text, .. }
            | BlockKind::Numbered { text, .. }
            | BlockKind::Todo { text, .. }
            | BlockKind::Quote { text }
            | BlockKind::Code { text, .. } => text.as_str(),
            BlockKind::Divider | BlockKind::Unknown => "",
        }
    }

    #[allow(dead_code)]
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match &mut self.kind {
            BlockKind::Paragraph { text }
            | BlockKind::Heading { text, .. }
            | BlockKind::Bullet { text, .. }
            | BlockKind::Numbered { text, .. }
            | BlockKind::Todo { text, .. }
            | BlockKind::Quote { text }
            | BlockKind::Code { text, .. } => Some(text),
            BlockKind::Divider | BlockKind::Unknown => None,
        }
    }

    pub fn set_text(&mut self, new_text: impl Into<String>) {
        if let Some(t) = self.text_mut() {
            *t = new_text.into();
        }
    }
}
