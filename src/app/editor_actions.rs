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
                let mut todo_count = 0;
                for block in &mut note.blocks {
                    if let crate::models::BlockKind::Todo { checked, .. } = &mut block.kind {
                        if todo_count == target_line_idx {
                            *checked = !*checked;
                            break;
                        }
                        todo_count += 1;
                    }
                }
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.plain_text();
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
                let new_block = match syntax {
                    "h1" => crate::models::Block::new(crate::models::BlockKind::Heading { level: 1, text: "Heading 1".to_string() }),
                    "h2" => crate::models::Block::new(crate::models::BlockKind::Heading { level: 2, text: "Heading 2".to_string() }),
                    "h3" => crate::models::Block::new(crate::models::BlockKind::Heading { level: 3, text: "Heading 3".to_string() }),
                    "bullet" => crate::models::Block::new(crate::models::BlockKind::Bullet { indent: 0, text: "Item point".to_string() }),
                    "number" => crate::models::Block::new(crate::models::BlockKind::Numbered { indent: 0, text: "Step item".to_string() }),
                    "quote" => crate::models::Block::new(crate::models::BlockKind::Quote { text: "Serenity in simplicity.".to_string() }),
                    "code" => crate::models::Block::new(crate::models::BlockKind::Code { language: "rust".to_string(), text: "fn main() {\n    println!(\"Simplicity\");\n}".to_string() }),
                    "checklist" => crate::models::Block::new(crate::models::BlockKind::Todo { checked: false, text: "New task item".to_string() }),
                    "divider" => crate::models::Block::new(crate::models::BlockKind::Divider),
                    _ => crate::models::Block::new(crate::models::BlockKind::Paragraph { text: "New text".to_string() }),
                };
                note.blocks.push(new_block);
                note.updated_at = chrono::Local::now();
                let _ = this_save(note, &self.storage);
                let body_str = note.plain_text();
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
