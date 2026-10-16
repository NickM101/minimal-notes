use crate::models::{ChecklistItem, VoiceRecording};
use crate::this_save;
use crate::NotesApp;
use gpui_kit::*;

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
        _window: &mut Window,
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
            let after_idx = if let Some(note) = self.notes.iter().find(|n| n.id == *id) {
                self.focused_block_id
                    .as_deref()
                    .and_then(|fid| note.blocks.iter().position(|b| b.id == fid))
            } else {
                return;
            };

            let kind = match syntax {
                "h1" => crate::models::BlockKind::Heading {
                    level: 1,
                    text: String::new(),
                },
                "h2" => crate::models::BlockKind::Heading {
                    level: 2,
                    text: String::new(),
                },
                "h3" => crate::models::BlockKind::Heading {
                    level: 3,
                    text: String::new(),
                },
                "bullet" => crate::models::BlockKind::Bullet {
                    indent: 0,
                    text: String::new(),
                },
                "number" => crate::models::BlockKind::Numbered {
                    indent: 0,
                    text: String::new(),
                },
                "quote" => crate::models::BlockKind::Quote {
                    text: String::new(),
                },
                "code" => crate::models::BlockKind::Code {
                    language: "rust".to_string(),
                    text: String::new(),
                },
                "checklist" => crate::models::BlockKind::Todo {
                    checked: false,
                    text: String::new(),
                },
                "divider" => crate::models::BlockKind::Divider,
                _ => crate::models::BlockKind::Paragraph {
                    text: String::new(),
                },
            };

            // The reducer inserts after `after_idx` (or appends when None) and
            // the FocusBlock effect focuses the new block.
            self.dispatch(
                crate::app::editor_reducer::EditorAction::InsertBlock { after_idx, kind },
                window,
                cx,
            );
        }
    }
}
