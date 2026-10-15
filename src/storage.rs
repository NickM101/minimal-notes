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

            let mut note5 = Note::new(
                "The Art of Quiet Software",
                "In an era characterized by relentless notifications, aggressive engagement metrics, and visual noise, quiet software represents a conscious return to intentional computing.\n\n\
                 ## Why Simplicity Endures\n\n\
                 The tools we choose to write with inevitably shape the thoughts we produce. When an application bombards you with flashing badges, excessive telemetry, and over-engineered toolbars, cognitive bandwidth is quietly drained before you even pen the first sentence.\n\n\
                 > \"Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away.\"\n\
                 > — Antoine de Saint-Exupéry\n\n\
                 ### The Three Pillars of Calm Interfaces\n\n\
                 1. **Zero Distraction Canvas**: The screen exists solely to elevate your words. Typography, line spacing, and margin proportions should evoke the quiet dignity of a well-typeset book.\n\
                 2. **Tactile Responsiveness**: Every interaction—a keypress, a checkbox click, or an icon hover—should feel purposeful and instant, devoid of sluggish network round-trips.\n\
                 3. **Local Sovereignty**: Your ideas belong on your machine. Simple plain-text and structured JSON formats outlive cloud platforms and subscription models.\n\n\
                 ```rust\n\
                 pub fn compose(thought: Thought) -> Clarity {\n\
                     thought.strip_distractions().crystallize()\n\
                 }\n\
                 ```\n\n\
                 When we build tools with respect for human attention, we make space for deep thinking, reflection, and lasting craft.",
            );
            note5.checklist = vec![
                ChecklistItem {
                    id: "chk-51".into(),
                    text: "Draft opening philosophy and motivation".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-52".into(),
                    text: "Outline the three pillars of calm interfaces".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-53".into(),
                    text: "Add code snippet and architectural reflection".into(),
                    checked: true,
                },
                ChecklistItem {
                    id: "chk-54".into(),
                    text: "Review typesetting and paragraph cadence".into(),
                    checked: false,
                },
                ChecklistItem {
                    id: "chk-55".into(),
                    text: "Publish article to personal essay archive".into(),
                    checked: false,
                },
            ];
            let _ = self.save(&note5);
            notes.push(note5);
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
