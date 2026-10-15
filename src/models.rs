use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceRecording {
    pub id: String,
    pub duration_seconds: u32,
    pub current_seconds: u32,
    pub is_playing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default)]
    pub images: Vec<String>, // file paths or identifiers
    #[serde(default)]
    pub voice: Option<VoiceRecording>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Note {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            body: body.into(),
            checklist: Vec::new(),
            images: Vec::new(),
            voice: None,
            created_at: now,
            updated_at: now,
        }
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
        let trimmed = self.body.trim();
        if trimmed.is_empty() {
            "No additional text".to_string()
        } else {
            // Filter markdown markers and join lines with space for smooth wrapping
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
        self.updated_at.format("%B %-d, %Y at %-I:%M %p").to_string()
    }
}
