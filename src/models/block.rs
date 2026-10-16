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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: String,
    pub kind: BlockKind,
    /// Unknown fields preserved from foreign/older schemas. serde's flatten
    /// + internally-tagged enum combination double-writes the tag and kind
    /// fields (breaking the next parse), so both directions are hand-rolled
    /// below: `extra` only ever holds keys `kind` does not claim.
    extra: HashMap<String, serde_json::Value>,
}

impl serde::Serialize for Block {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        let mut obj = serde_json::Map::new();
        obj.insert("id".into(), serde_json::Value::String(self.id.clone()));
        let mut kind_fields = match serde_json::to_value(&self.kind).map_err(Error::custom)? {
            serde_json::Value::Object(fields) => fields,
            _ => serde_json::Map::new(),
        };
        // Unknown blocks keep their original foreign tag as data instead of
        // the "unknown" placeholder the enum parses into.
        if self.kind == BlockKind::Unknown {
            if let Some(tag) = self.extra.get("type") {
                kind_fields.insert("type".into(), tag.clone());
            }
        }
        obj.extend(kind_fields);
        // Preserve unknown extras without ever shadowing a known field.
        for (k, v) in &self.extra {
            obj.entry(k.clone()).or_insert_with(|| v.clone());
        }
        serde_json::Value::Object(obj).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Block {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let mut obj = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        let id = match obj.remove("id") {
            Some(serde_json::Value::String(s)) => s,
            _ => return Err(D::Error::missing_field("id")),
        };
        let kind: BlockKind = serde_json::from_value(serde_json::Value::Object(obj.clone()))
            .map_err(Error::custom)?;
        // Whatever the kind consumed is not "unknown"; drop it from extras —
        // except an unrecognized tag, which stays so the original type name
        // survives a round-trip.
        let is_unknown = matches!(kind, BlockKind::Unknown);
        if let Ok(serde_json::Value::Object(known)) = serde_json::to_value(&kind) {
            for key in known.keys() {
                if is_unknown && key == "type" {
                    continue;
                }
                obj.remove(key);
            }
        }
        Ok(Block {
            id,
            kind,
            extra: obj.into_iter().collect(),
        })
    }
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

    /// Fields preserved from foreign/older schemas that `kind` doesn't model.
    #[allow(dead_code)] // public API; exercised by the integration tests
    pub fn extra(&self) -> &HashMap<String, serde_json::Value> {
        &self.extra
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A block that has been deserialized once must serialize to JSON that
    /// parses identically again — forever. Regressed once: serde's flatten
    /// map double-wrote the tag and kind fields after the first parse,
    /// corrupting every note on its next save.
    #[test]
    fn serde_roundtrip_is_stable_with_unknown_fields() {
        let json = r#"{
            "id": "b1",
            "type": "paragraph",
            "text": "hello",
            "legacy_field": {"deep": true}
        }"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.text(), "hello");

        let first = serde_json::to_string(&block).unwrap();
        let again: Block = serde_json::from_str(&first).unwrap();
        let second = serde_json::to_string(&again).unwrap();
        assert_eq!(first, second, "round-trip must be idempotent");

        // The preserved unknown field survives both directions…
        let parsed: serde_json::Value = serde_json::from_str(&second).unwrap();
        assert_eq!(
            parsed.get("legacy_field"),
            Some(&serde_json::json!({"deep": true})),
        );
        // …and no field is ever written twice.
        let keys: Vec<&str> = parsed
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        let mut unique = keys.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            keys.len(),
            unique.len(),
            "duplicate keys would break this assert"
        );
    }

    #[test]
    fn serde_roundtrip_covers_every_kind() {
        let kinds = vec![
            BlockKind::Paragraph { text: "p".into() },
            BlockKind::Heading {
                level: 2,
                text: "h".into(),
            },
            BlockKind::Bullet {
                text: "b".into(),
                indent: 3,
            },
            BlockKind::Numbered {
                text: "n".into(),
                indent: 0,
            },
            BlockKind::Todo {
                text: "t".into(),
                checked: true,
            },
            BlockKind::Quote { text: "q".into() },
            BlockKind::Code {
                language: "go".into(),
                text: "func() {}".into(),
            },
            BlockKind::Divider,
            BlockKind::Unknown,
        ];
        for kind in kinds {
            let block = Block::new(kind);
            let json = serde_json::to_string(&block).unwrap();
            let back: Block = serde_json::from_str(&json).unwrap();
            assert_eq!(back.kind, block.kind);
            assert_eq!(back.id, block.id);
            assert_eq!(serde_json::to_string(&back).unwrap(), json);
        }
    }
}
