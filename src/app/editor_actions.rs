use gpui_kit::*;
use crate::models::{ChecklistItem, VoiceRecording};
use crate::this_save;
use crate::NotesApp;

impl NotesApp {
    pub fn toggle_checklist_item(&mut self, item_id: &str, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                if let Some(item) = note.checklist.iter_mut().find(|i| i.id == item_id) {
                    item.checked = !item.checked;
                    note.updated_at = chrono::Local::now();
                    let _ = this_save(note, &self.storage);
                    cx.notify();
                }
            }
        }
    }

    pub fn toggle_markdown_checkbox(
        &mut self,
        target_line_idx: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let lines: Vec<&str> = note.body.lines().collect();
                let mut new_lines = Vec::with_capacity(lines.len());
                for (idx, line) in lines.iter().enumerate() {
                    if idx == target_line_idx {
                        let trimmed = line.trim_start();
                        let indent = &line[..(line.len() - trimmed.len())];
                        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
                            new_lines.push(format!("{}- [x] {}", indent, rest));
                        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
                            new_lines.push(format!("{}- [ ] {}", indent, rest));
                        } else if let Some(rest) = trimmed.strip_prefix("- [X] ") {
                            new_lines.push(format!("{}- [ ] {}", indent, rest));
                        } else {
                            new_lines.push(line.to_string());
                        }
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                note.body = new_lines.join("\n");
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.body.clone();
                self.is_syncing = true;
                self.editor_state.update(cx, |s, cx| {
                    s.set_value(&body_str, window, cx);
                });
                self.is_syncing = false;
                cx.notify();
            }
        }
    }

    pub fn add_checklist_item(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let count = note.checklist.len() + 1;
                note.checklist.push(ChecklistItem {
                    id: format!("chk-{}", uuid::Uuid::new_v4()),
                    text: format!("Task item {}", count),
                    checked: false,
                });
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn remove_checklist_item(&mut self, item_id: &str, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.checklist.retain(|i| i.id != item_id);
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn toggle_voice_player(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                if let Some(voice) = &mut note.voice {
                    voice.is_playing = !voice.is_playing;
                    note.updated_at = chrono::Local::now();
                    let _ = this_save(note, &self.storage);
                    cx.notify();
                }
            }
        }
    }

    pub fn remove_voice_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.voice = None;
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                cx.notify();
            }
        }
    }

    pub fn add_sample_voice_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.voice = Some(VoiceRecording {
                    id: format!("rec-{}", uuid::Uuid::new_v4()),
                    duration_seconds: 48,
                    current_seconds: 14,
                    is_playing: false,
                });
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                self.show_add_modal = false;
                cx.notify();
            }
        }
    }

    pub fn add_sample_image(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.images.push("zen_art".to_string());
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                self.show_add_modal = false;
                cx.notify();
            }
        }
    }

    pub fn insert_formatting(&mut self, syntax: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                let current_body = &note.body;
                let prepend_newline = !current_body.is_empty() && !current_body.ends_with('\n');
                let prefix = if prepend_newline { "\n" } else { "" };

                match syntax {
                    "h1" => note.body.push_str(&format!("{}# Heading 1\n", prefix)),
                    "h2" => note.body.push_str(&format!("{}## Heading 2\n", prefix)),
                    "h3" => note.body.push_str(&format!("{}### Heading 3\n", prefix)),
                    "bold" => note.body.push_str("**bold text**"),
                    "italic" => note.body.push_str("*italic thought*"),
                    "underline" => note.body.push_str("<u>underlined text</u>"),
                    "highlight" => note.body.push_str("==highlighted note=="),
                    "strike" => note.body.push_str("~~struck text~~"),
                    "code_inline" => note.body.push_str("`code snippet`"),
                    "bullet" => note.body.push_str(&format!("{}- Item point\n", prefix)),
                    "number" => note.body.push_str(&format!("{}1. Step item\n", prefix)),
                    "quote" => note.body.push_str(&format!("{}> Serenity in simplicity.\n", prefix)),
                    "callout" => note.body.push_str(&format!("{}> [!NOTE] Zen is not a destination, it is an approach.\n", prefix)),
                    "code" => note.body.push_str(&format!("{}```rust\nfn main() {{\n    println!(\"Simplicity\");\n}}\n```\n", prefix)),
                    "checklist" => {
                        note.body.push_str(&format!("{}- [ ] New task item\n", prefix));
                    }
                    "divider" => note.body.push_str(&format!("{}---\n", prefix)),
                    _ => {}
                }
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.body.clone();
                self.is_syncing = true;
                self.editor_state.update(cx, |s, cx| {
                    s.set_value(&body_str, window, cx);
                });
                self.is_syncing = false;
                cx.notify();
            }
        }
    }


}
