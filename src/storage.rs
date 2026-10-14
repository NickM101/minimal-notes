use std::fs;
use std::path::PathBuf;
use crate::models::{ChecklistItem, Note, VoiceRecording};

pub struct Storage {
    dir: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let dir = PathBuf::from("notes_data");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        Self { dir }
    }

    pub fn load_all(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(note) = serde_json::from_str::<Note>(&content) {
                            notes.push(note);
                        }
                    }
                }
            }
        }

        // Sort descending by updated_at
        notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        if notes.is_empty() {
            // Seed inspiring notes matching the inspo Behance design
            let mut note1 = Note::new(
                "Minimalism",
                "First I admit that I am not a web designer, but a design lover.\n\n\
                 Minimalism today refers to a certain style and applies in many fields: \
                 Art, architecture, literature, philosophy, law, as way of life.\n\n\
                 Are you a designer who hides behind ornament and decoration? \
                 Or the other one who strips his work down to its most fundamental features functionally?",
            );
            note1.checklist = vec![
                ChecklistItem {
                    id: "chk-1".into(),
                    text: "I have read about that topic.".into(),
                    checked: false,
                },
                ChecklistItem {
                    id: "chk-2".into(),
                    text: "I have read about that topic.".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-3".into(),
                    text: "I have read about that topic.".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-4".into(),
                    text: "I have read about that topic.".into(),
                    checked: false,
                },
            ];
            note1.voice = Some(VoiceRecording {
                id: "rec-1".into(),
                duration_seconds: 64,
                current_seconds: 18,
                is_playing: false,
            });
            let _ = self.save(&note1);
            notes.push(note1);

            let mut note2 = Note::new(
                "New way to think",
                "My plan for today:\n\n\
                 Take notes in a variety of formats including text, photos, voice recordings and more.\n\
                 Focus on what matters most and eliminate unnecessary clutter.",
            );
            note2.checklist = vec![
                ChecklistItem {
                    id: "chk-21".into(),
                    text: "Call mom".into(),
                    checked: false,
                },
                ChecklistItem {
                    id: "chk-22".into(),
                    text: "Book flight to Morocco".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-23".into(),
                    text: "Preview wireframe, send feedback to designer".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-24".into(),
                    text: "Make dinner reservation".into(),
                    checked: true,
                },
            ];
            let _ = self.save(&note2);
            notes.push(note2);

            let note3 = Note::new(
                "Whisper",
                "Aiming to be an app that inspires people to take notes in a simple and efficient way.\n\
                 Notty is inspired by the Zen design to have obvious interfaces which lead to great user experience.",
            );
            let _ = self.save(&note3);
            notes.push(note3);

            let note4 = Note::new(
                "Day 1",
                "Inspired from old notes and books. The light Green-Brown color refers to paper, and black color presents the ink.",
            );
            let _ = self.save(&note4);
            notes.push(note4);
        }

        notes
    }

    pub fn save(&self, note: &Note) -> std::io::Result<()> {
        let file_path = self.dir.join(format!("{}.json", note.id));
        let json = serde_json::to_string_pretty(note)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> std::io::Result<()> {
        let file_path = self.dir.join(format!("{}.json", id));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }
}
