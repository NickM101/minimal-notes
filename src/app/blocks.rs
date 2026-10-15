use std::time::Instant;
use gpui_kit::*;
use gpui_kit::component::input::{InputEvent, InputState};
use crate::NotesApp;

impl NotesApp {
    pub fn sync_block_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self._block_subscriptions.clear();
        self.block_input_states.clear();

        let note = match self.selected_note().cloned() {
            Some(n) => n,
            None => return,
        };

        for block in &note.blocks {
            let block_id = block.id.clone();
            let initial_text = block.text().to_string();
            let placeholder_str = match &block.kind {
                crate::models::BlockKind::Paragraph { .. } => "Type '/' for commands...",
                crate::models::BlockKind::Heading { .. } => "Heading...",
                crate::models::BlockKind::Bullet { .. } | crate::models::BlockKind::Numbered { .. } => "List item...",
                crate::models::BlockKind::Todo { .. } => "To-do...",
                crate::models::BlockKind::Quote { .. } => "Empty quote...",
                crate::models::BlockKind::Code { .. } => "Code...",
                _ => "",
            };

            let input_state = cx.new(|cx| {
                let mut state = InputState::new(window, cx).placeholder(placeholder_str);
                state.set_value(&initial_text, window, cx);
                state
            });

            let bid_clone = block_id.clone();
            let sub = cx.subscribe_in(&input_state, window, move |this, state, event, window, cx| {
                if this.is_syncing {
                    return;
                }
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.handle_block_enter(&bid_clone, window, cx);
                    return;
                }
                if matches!(event, InputEvent::Change) {
                    let new_text = state.read(cx).value().to_string();

                    // Check if input begins with markdown shortcut trigger
                    if this.try_convert_markdown_shortcut(&bid_clone, &new_text, window, cx) {
                        this.slash_menu_target = None;
                        return;
                    }

                    // Check for slash menu trigger (e.g., text starts with "/" or ends with "/")
                    if let Some(query) = new_text.strip_prefix('/') {
                        this.slash_menu_target = Some((bid_clone.clone(), query.to_string()));
                    } else {
                        if this.slash_menu_target.as_ref().map_or(false, |(id, _)| id == &bid_clone) {
                            this.slash_menu_target = None;
                        }
                    }

                    if let Some(id) = &this.selected_id {
                        if let Some(note) = this.notes.iter_mut().find(|n| n.id == *id) {
                            if let Some(blk) = note.blocks.iter_mut().find(|b| b.id == bid_clone) {
                                if let Some(text_ref) = blk.text_mut() {
                                    *text_ref = new_text;
                                    note.updated_at = chrono::Local::now();
                                    this.schedule_save(cx);
                                    cx.notify();
                                }
                            }
                        }
                    }
                }
            });

            self._block_subscriptions.push(sub);
            self.block_input_states.push((block_id, input_state));
        }
    }

    pub fn handle_block_enter(
        &mut self,
        block_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        // Find the block index and current block info
        let (idx, current_kind) = {
            let note = match self.notes.iter().find(|n| n.id == note_id) {
                Some(n) => n,
                None => return,
            };
            match note.blocks.iter().position(|b| b.id == block_id) {
                Some(i) => (i, note.blocks[i].kind.clone()),
                None => return,
            }
        };

        // Determine split point from cursor
        let (head_text, tail_text) = if let Some((_, state)) = self.block_input_states.iter().find(|(id, _)| id == block_id) {
            let text = state.read(cx).value().to_string();
            let cursor = state.read(cx).cursor().min(text.len());
            // Safe UTF-8 character boundary slice
            let char_indices: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
            let split_pos = if cursor >= text.len() {
                text.len()
            } else if char_indices.contains(&cursor) {
                cursor
            } else {
                char_indices.iter().copied().rfind(|&i| i <= cursor).unwrap_or(0)
            };
            (text[..split_pos].to_string(), text[split_pos..].to_string())
        } else {
            (String::new(), String::new())
        };

        // Determine the next block kind
        let new_block_kind = match current_kind {
            crate::models::BlockKind::Bullet { indent, .. } => {
                crate::models::BlockKind::Bullet {
                    text: tail_text,
                    indent,
                }
            }
            crate::models::BlockKind::Numbered { indent, .. } => {
                crate::models::BlockKind::Numbered {
                    text: tail_text,
                    indent,
                }
            }
            crate::models::BlockKind::Todo { .. } => {
                crate::models::BlockKind::Todo {
                    text: tail_text,
                    checked: false,
                }
            }
            _ => crate::models::BlockKind::Paragraph {
                text: tail_text,
            },
        };

        let new_block = crate::models::Block::new(new_block_kind);
        let new_block_id = new_block.id.clone();

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if let Some(curr_blk) = note.blocks.get_mut(idx) {
                curr_blk.set_text(head_text);
            }
            note.blocks.insert(idx + 1, new_block);
            note.updated_at = chrono::Local::now();
            let _ = self.storage.save(note);
        }

        self.sync_block_inputs(window, cx);

        // Focus the newly inserted block
        if let Some((_, next_state)) = self.block_input_states.iter().find(|(id, _)| id == &new_block_id) {
            next_state.update(cx, |s, cx| {
                s.focus(window, cx);
            });
        }

        cx.notify();
    }

    pub fn handle_block_backspace(
        &mut self,
        block_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        // Check cursor position: only merge if at start of block (cursor == 0)
        let is_at_start = if let Some((_, state)) = self.block_input_states.iter().find(|(id, _)| id == block_id) {
            state.read(cx).cursor() == 0
        } else {
            false
        };

        if !is_at_start {
            return;
        }

        let note = match self.notes.iter().find(|n| n.id == note_id) {
            Some(n) => n,
            None => return,
        };

        let idx = match note.blocks.iter().position(|b| b.id == block_id) {
            Some(i) => i,
            None => return,
        };

        // If it's a non-paragraph block and empty or at start, turn it back into a paragraph first!
        let curr_kind = &note.blocks[idx].kind;
        let is_non_p = !matches!(curr_kind, crate::models::BlockKind::Paragraph { .. });
        if is_non_p {
            let curr_text = note.blocks[idx].text().to_string();
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
                note.blocks[idx].kind = crate::models::BlockKind::Paragraph { text: curr_text };
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            }
            self.sync_block_inputs(window, cx);
            if let Some((_, state)) = self.block_input_states.iter().find(|(id, _)| id == block_id) {
                state.update(cx, |s, cx| {
                    s.focus(window, cx);
                });
            }
            cx.notify();
            return;
        }

        // If this is the very first block, cannot merge with previous
        if idx == 0 {
            return;
        }

        let prev_block_id = note.blocks[idx - 1].id.clone();
        let curr_text = note.blocks[idx].text().to_string();

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            let prev_text = note.blocks[idx - 1].text().to_string();
            note.blocks[idx - 1].set_text(format!("{}{}", prev_text, curr_text));
            note.blocks.remove(idx);
            note.updated_at = chrono::Local::now();
            let _ = self.storage.save(note);
        }

        self.sync_block_inputs(window, cx);

        // Focus previous block
        if let Some((_, prev_state)) = self.block_input_states.iter().find(|(id, _)| id == &prev_block_id) {
            prev_state.update(cx, |s, cx| {
                s.focus(window, cx);
            });
        }

        cx.notify();
    }

    pub fn handle_block_arrow(
        &mut self,
        block_id: &str,
        is_up: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        let note = match self.notes.iter().find(|n| n.id == note_id) {
            Some(n) => n,
            None => return,
        };

        let idx = match note.blocks.iter().position(|b| b.id == block_id) {
            Some(i) => i,
            None => return,
        };

        if is_up {
            // Move focus to previous block if exists
            if idx > 0 {
                let target_id = note.blocks[idx - 1].id.clone();
                if let Some((_, target_state)) = self.block_input_states.iter().find(|(id, _)| id == &target_id) {
                    target_state.update(cx, |s, cx| {
                        s.focus(window, cx);
                    });
                    cx.notify();
                }
            } else {
                // At very first block, Up arrow moves focus to the note title!
                self.title_state.update(cx, |s, cx| {
                    s.focus(window, cx);
                });
                cx.notify();
            }
        } else {
            // Move focus to next block if exists
            if idx + 1 < note.blocks.len() {
                let target_id = note.blocks[idx + 1].id.clone();
                if let Some((_, target_state)) = self.block_input_states.iter().find(|(id, _)| id == &target_id) {
                    target_state.update(cx, |s, cx| {
                        s.focus(window, cx);
                    });
                    cx.notify();
                }
            }
        }
    }

    pub fn handle_block_tab(
        &mut self,
        block_id: &str,
        is_shift: bool,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if let Some(blk) = note.blocks.iter_mut().find(|b| b.id == block_id) {
                match &mut blk.kind {
                    crate::models::BlockKind::Bullet { indent, .. }
                    | crate::models::BlockKind::Numbered { indent, .. } => {
                        if is_shift {
                            if *indent > 0 {
                                *indent -= 1;
                            }
                        } else {
                            if *indent < 5 {
                                *indent += 1;
                            }
                        }
                        note.updated_at = chrono::Local::now();
                        let _ = self.storage.save(note);
                        cx.notify();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn try_convert_markdown_shortcut(
        &mut self,
        block_id: &str,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // Only trigger if text matches a prefix + space/trigger
        let (new_kind, _remaining_text) = if let Some(rest) = text.strip_prefix("### ") {
            (crate::models::BlockKind::Heading { level: 3, text: rest.to_string() }, rest)
        } else if let Some(rest) = text.strip_prefix("## ") {
            (crate::models::BlockKind::Heading { level: 2, text: rest.to_string() }, rest)
        } else if let Some(rest) = text.strip_prefix("# ") {
            (crate::models::BlockKind::Heading { level: 1, text: rest.to_string() }, rest)
        } else if let Some(rest) = text.strip_prefix("- [ ] ") {
            (crate::models::BlockKind::Todo { text: rest.to_string(), checked: false }, rest)
        } else if let Some(rest) = text.strip_prefix("- [x] ") {
            (crate::models::BlockKind::Todo { text: rest.to_string(), checked: true }, rest)
        } else if let Some(rest) = text.strip_prefix("[] ") {
            (crate::models::BlockKind::Todo { text: rest.to_string(), checked: false }, rest)
        } else if let Some(rest) = text.strip_prefix("- ") {
            (crate::models::BlockKind::Bullet { text: rest.to_string(), indent: 0 }, rest)
        } else if let Some(rest) = text.strip_prefix("* ") {
            (crate::models::BlockKind::Bullet { text: rest.to_string(), indent: 0 }, rest)
        } else if let Some(rest) = text.strip_prefix("1. ") {
            (crate::models::BlockKind::Numbered { text: rest.to_string(), indent: 0 }, rest)
        } else if let Some(rest) = text.strip_prefix("> ") {
            (crate::models::BlockKind::Quote { text: rest.to_string() }, rest)
        } else if text.starts_with("```") {
            let after_ticks = &text[3..];
            let lang = after_ticks.trim();
            (
                crate::models::BlockKind::Code {
                    language: if lang.is_empty() { "rust".to_string() } else { lang.to_string() },
                    text: String::new(),
                },
                "",
            )
        } else if text == "---" {
            (crate::models::BlockKind::Divider, "")
        } else {
            return false;
        };

        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return false,
        };

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if let Some(blk) = note.blocks.iter_mut().find(|b| b.id == block_id) {
                blk.kind = new_kind;
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            }
        }

        self.sync_block_inputs(window, cx);

        // Keep focus in the converted block
        if let Some((_, state)) = self.block_input_states.iter().find(|(id, _)| id == block_id) {
            state.update(cx, |s, cx| {
                s.focus(window, cx);
            });
        }

        cx.notify();
        true
    }

    pub fn convert_block_type(
        &mut self,
        block_id: &str,
        new_kind_factory: impl FnOnce(String) -> crate::models::BlockKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        let curr_text = if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            note.blocks.iter().find(|b| b.id == block_id).map(|b| b.text().to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        let new_kind = new_kind_factory(curr_text);

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if let Some(blk) = note.blocks.iter_mut().find(|b| b.id == block_id) {
                blk.kind = new_kind;
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            }
        }

        self.slash_menu_target = None;
        self.sync_block_inputs(window, cx);

        if let Some((_, state)) = self.block_input_states.iter().find(|(id, _)| id == block_id) {
            state.update(cx, |s, cx| {
                s.focus(window, cx);
            });
        }

        cx.notify();
    }

    pub fn delete_block(
        &mut self,
        block_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if note.blocks.len() > 1 {
                note.blocks.retain(|b| b.id != block_id);
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            } else if let Some(first) = note.blocks.first_mut() {
                first.kind = crate::models::BlockKind::Paragraph { text: String::new() };
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            }
        }

        self.slash_menu_target = None;
        self.sync_block_inputs(window, cx);
        cx.notify();
    }

    pub fn duplicate_block(
        &mut self,
        block_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            if let Some(idx) = note.blocks.iter().position(|b| b.id == block_id) {
                let copy = crate::models::Block::new(note.blocks[idx].kind.clone());
                note.blocks.insert(idx + 1, copy);
                note.updated_at = chrono::Local::now();
                let _ = self.storage.save(note);
            }
        }

        self.sync_block_inputs(window, cx);
        cx.notify();
    }

    pub fn schedule_save(&mut self, cx: &mut Context<Self>) {
        let storage = self.storage.clone();

        self._save_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(500))
                .await;

            let _ = this.update(cx, |app, cx| {
                if let Some(note) = app.selected_note().cloned() {
                    if storage.save(&note).is_ok() {
                        app.status_message = Some(("Saved".to_string(), Instant::now()));
                        cx.notify();
                    }
                }
            });
        }));
    }
}
