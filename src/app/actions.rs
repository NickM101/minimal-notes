use std::time::Instant;
use gpui_kit::*;
use crate::models::Note;
use crate::theme::CurrentScreen;
use crate::NotesApp;

impl NotesApp {
    pub fn open_note(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_id = Some(id.to_string());
        if let Some(note) = self.notes.iter().find(|n| n.id == id).cloned() {
            self.is_syncing = true;
            self.title_state.update(cx, |s, cx| {
                s.set_value(&note.title, window, cx);
            });
            self.editor_state.update(cx, |s, cx| {
                s.set_value(&note.plain_text(), window, cx);
            });
            self.is_syncing = false;
        }
        self.sync_block_inputs(window, cx);
        self.current_screen = CurrentScreen::Editor;
        cx.notify();
    }

    pub fn create_new_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_note = Note::new("", "");
        let _ = self.storage.save(&new_note);
        let id = new_note.id.clone();
        self.notes.insert(0, new_note);

        self.selected_id = Some(id);
        self.is_syncing = true;
        self.title_state.update(cx, |s, cx| {
            s.set_value("", window, cx);
        });
        self.editor_state.update(cx, |s, cx| {
            s.set_value("", window, cx);
        });
        self.is_syncing = false;
        self.sync_block_inputs(window, cx);
        self.current_screen = CurrentScreen::Editor;
        cx.notify();
    }

    pub fn delete_selected_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_id.clone() {
            self.delete_note_by_id(&id, window, cx);
        }
    }

    pub fn delete_note_by_id(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.storage.delete(id);
        self.notes.retain(|n| n.id != id);

        if self.selected_id.as_deref() == Some(id) {
            if let Some(first) = self.notes.first().cloned() {
                self.open_note(&first.id, window, cx);
            } else {
                self.selected_id = None;
                self.current_screen = CurrentScreen::Browse;
            }
        }
        self.status_message = Some(("Note deleted".to_string(), Instant::now()));
        cx.notify();
    }

    pub fn duplicate_note(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(orig) = self.notes.iter().find(|n| n.id == id).cloned() {
            let mut copy = orig.clone();
            copy.id = uuid::Uuid::new_v4().to_string();
            copy.title = if copy.title.is_empty() {
                "Copy of Untitled".to_string()
            } else {
                format!("{} (Copy)", copy.title)
            };
            copy.created_at = chrono::Local::now();
            copy.updated_at = chrono::Local::now();
            let _ = self.storage.save(&copy);
            let new_id = copy.id.clone();
            self.notes.insert(0, copy);
            self.open_note(&new_id, window, cx);
            self.status_message = Some(("Note duplicated".to_string(), Instant::now()));
            cx.notify();
        }
    }

    pub fn trigger_save(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.selected_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == *id) {
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
                self.status_message = Some(("Saved".to_string(), Instant::now()));
                cx.notify();
            }
        }
    }
}
