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
        let trimmed = self.body.trim();
        if trimmed.is_empty() {
            "No additional text".to_string()
        } else {
            let first_line = trimmed.lines().next().unwrap_or("");
            if first_line.chars().count() > 60 {
                let s: String = first_line.chars().take(57).collect();
                format!("{}...", s)
            } else {
                first_line.to_string()
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
