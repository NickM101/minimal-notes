use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VoiceRecording {
    pub id: String,
    pub duration_seconds: u32,
    pub current_seconds: u32,
    pub is_playing: bool,
}
