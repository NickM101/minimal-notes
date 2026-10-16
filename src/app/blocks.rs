use crate::app::editor_reducer::{self, EditorAction, EditorEffect};
use crate::NotesApp;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::*;
use std::time::Instant;

/// A block's live text input and its subscription, created together and
/// dropped together. Handles live for the lifetime of their block — they are
/// never wholesale-recreated, so gpui's per-state undo history and cursor
/// survive structural edits (the core idea ported from cascade-editor).
pub struct BlockInputHandle {
    pub block_id: String,
    pub placeholder_key: &'static str,
    pub state: Entity<InputState>,
    _subscription: Subscription,
}

impl NotesApp {
    /// The input state of the FIRST block in document order (handles are
    /// stored in creation order, not document order).
    pub fn first_block_state(&self) -> Option<Entity<InputState>> {
        let first_id = self.selected_note()?.blocks.first()?.id.clone();
        self.block_input_states
            .iter()
            .find(|h| h.block_id == first_id)
            .map(|h| h.state.clone())
    }

    /// The input state of the LAST block in document order.
    pub fn last_block_state(&self) -> Option<Entity<InputState>> {
        let last_id = self.selected_note()?.blocks.last()?.id.clone();
        self.block_input_states
            .iter()
            .find(|h| h.block_id == last_id)
            .map(|h| h.state.clone())
    }

    /// The only place a block input + subscription is ever created.
    fn create_block_input(
        &mut self,
        block: &crate::models::Block,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> BlockInputHandle {
        let block_id = block.id.clone();
        let placeholder = editor_reducer::placeholder_for_kind(&block.kind);
        let placeholder_key = editor_reducer::kind_key(&block.kind);
        let initial_text = block.text().to_string();

        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder(placeholder);
            state.set_value(&initial_text, window, cx);
            state
        });

        let bid_change = block_id.clone();
        let bid_focus = block_id.clone();
        let bid_blur = block_id.clone();
        let sub = cx.subscribe_in(
            &input_state,
            window,
            move |this, state, event, window, cx| {
                if this.is_syncing {
                    return;
                }
                match event {
                    InputEvent::Focus => {
                        this.focused_block_id = Some(bid_focus.clone());
                        cx.notify();
                    }
                    InputEvent::Blur => {
                        if this.focused_block_id.as_deref() == Some(&bid_blur) {
                            this.focused_block_id = None;
                            cx.notify();
                        }
                    }
                    InputEvent::PressEnter {
                        secondary: false, ..
                    } => {
                        // Slash menu open for this block: Enter executes the
                        // highlighted command instead of splitting. execute_slash_command
                        // reads the query from slash_menu_target itself, so don't clear it first.
                        if this
                            .slash_menu_target
                            .as_ref()
                            .map_or(false, |(id, _)| id == &bid_focus)
                        {
                            let (_, query) = this.slash_menu_target.clone().unwrap();
                            let selection = this.slash_menu_selection.take().unwrap_or(0);
                            let filtered =
                                crate::views::editor::slash_menu::filtered_slash_command_ids(
                                    &query,
                                );
                            if let Some(&cmd) = filtered.get(selection) {
                                crate::views::editor::slash_menu::execute_slash_command(
                                    this, cmd, &bid_focus, window, cx,
                                );
                            } else {
                                // Nothing matches the filter — just close the menu
                                this.slash_menu_target = None;
                            }
                            this.slash_menu_selection = None;
                            cx.notify();
                            return;
                        }
                        let text = state.read(cx).value().to_string();
                        let cursor = state.read(cx).cursor().min(text.len());
                        this.dispatch(
                            EditorAction::SplitBlock {
                                id: bid_focus.clone(),
                                split_pos: cursor,
                            },
                            window,
                            cx,
                        );
                    }
                    InputEvent::Change => {
                        let new_text = state.read(cx).value().to_string();

                        // Slash menu trigger: text starts with "/"
                        if let Some(query) = new_text.strip_prefix('/') {
                            this.slash_menu_target = Some((bid_change.clone(), query.to_string()));
                            this.slash_menu_selection = Some(0);
                        } else if this
                            .slash_menu_target
                            .as_ref()
                            .map_or(false, |(id, _)| id == &bid_change)
                        {
                            this.slash_menu_target = None;
                            this.slash_menu_selection = None;
                        }

                        // Markdown shortcut: "# " converts the block in place.
                        // Only on a typing-sized change — never on a paste, which
                        // may legitimately begin with "# " or "```" and must be
                        // preserved verbatim. Code blocks hold literal text and
                        // never convert.
                        let mut shortcut_kind = None;
                        if let Some(id) = &this.selected_id {
                            if let Some(note) = this.notes.iter().find(|n| n.id == *id) {
                                if let Some(blk) = note.blocks.iter().find(|b| b.id == bid_change) {
                                    let is_code =
                                        matches!(blk.kind, crate::models::BlockKind::Code { .. });
                                    if !is_code {
                                        shortcut_kind = editor_reducer::typing_markdown_shortcut(
                                            blk.text(),
                                            &new_text,
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(kind) = shortcut_kind {
                            this.dispatch(
                                EditorAction::ConvertBlock {
                                    id: bid_change.clone(),
                                    kind,
                                },
                                window,
                                cx,
                            );
                            return;
                        }

                        if let Some(id) = &this.selected_id {
                            if let Some(note) = this.notes.iter_mut().find(|n| n.id == *id) {
                                if let Some(blk) =
                                    note.blocks.iter_mut().find(|b| b.id == bid_change)
                                {
                                    if blk.text() != new_text {
                                        blk.set_text(new_text);
                                        note.updated_at = chrono::Local::now();
                                        this.schedule_save(cx);
                                        cx.notify();
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            },
        );

        BlockInputHandle {
            block_id,
            placeholder_key,
            state: input_state,
            _subscription: sub,
        }
    }

    /// Diffs the live input handles against the selected note's blocks,
    /// creating handles only for genuinely new blocks and updating
    /// placeholders when a block's kind changed. Existing states — with their
    /// undo history and cursors — are left untouched.
    pub fn reconcile_block_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let blocks = match self.selected_note() {
            Some(n) => n.blocks.clone(),
            None => {
                self.block_input_states.clear();
                return;
            }
        };

        // Drop handles whose block no longer exists.
        self.block_input_states
            .retain(|h| blocks.iter().any(|b| b.id == h.block_id));

        // Create handles for new blocks.
        let missing: Vec<crate::models::Block> = blocks
            .iter()
            .filter(|b| !self.block_input_states.iter().any(|h| h.block_id == b.id))
            .cloned()
            .collect();
        for block in &missing {
            let handle = self.create_block_input(block, window, cx);
            self.block_input_states.push(handle);
        }

        // Refresh placeholders for blocks whose kind changed.
        for block in &blocks {
            let key = editor_reducer::kind_key(&block.kind);
            let needs_update = self
                .block_input_states
                .iter()
                .find(|h| h.block_id == block.id)
                .map_or(false, |h| h.placeholder_key != key);
            if needs_update {
                if let Some(handle) = self
                    .block_input_states
                    .iter_mut()
                    .find(|h| h.block_id == block.id)
                {
                    handle.placeholder_key = key;
                    let placeholder = editor_reducer::placeholder_for_kind(&block.kind);
                    handle.state.update(cx, |s, cx| {
                        s.set_placeholder(placeholder, window, cx);
                    });
                }
            }
        }
    }

    /// Single entry point for every document mutation: reduce, reconcile,
    /// sync texts, apply focus/save effects. (Port of cascade-editor's
    /// action → reducer → effect pipeline.)
    pub fn dispatch(&mut self, action: EditorAction, window: &mut Window, cx: &mut Context<Self>) {
        let note_id = match &self.selected_id {
            Some(id) => id.clone(),
            None => return,
        };
        let mut blocks = match self.notes.iter().find(|n| n.id == note_id) {
            Some(n) => n.blocks.clone(),
            None => return,
        };

        let effects = editor_reducer::reduce(&mut blocks, &action);

        if let Some(note) = self.notes.iter_mut().find(|n| n.id == note_id) {
            note.blocks = blocks;
            note.updated_at = chrono::Local::now();
        }

        self.reconcile_block_inputs(window, cx);

        // Push reducer-driven text changes into the surviving input states.
        // replace_all preserves undo and leaves the caret at the end of the
        // new text, which is what merges and conversions want.
        self.is_syncing = true;
        let note_blocks = self
            .selected_note()
            .map(|n| n.blocks.clone())
            .unwrap_or_default();
        for handle in &self.block_input_states {
            let model_text = match note_blocks.iter().find(|b| b.id == handle.block_id) {
                Some(b) => b.text().to_string(),
                None => continue,
            };
            let current = handle.state.read(cx).value().to_string();
            if current != model_text {
                handle.state.update(cx, |s, cx| {
                    s.replace_all(model_text, window, cx);
                });
            }
        }
        self.is_syncing = false;

        // Apply effects: focus first (before pruning drops any dead states).
        let mut should_save = false;
        for effect in effects {
            match effect {
                EditorEffect::FocusBlock { id, cursor } => {
                    if let Some(handle) = self.block_input_states.iter().find(|h| h.block_id == id)
                    {
                        let state = handle.state.clone();
                        state.update(cx, |s, cx| {
                            s.focus(window, cx);
                            if let Some(pos) = cursor {
                                s.set_selected_range(pos..pos, cx);
                            }
                        });
                        self.focused_block_id = Some(id);
                    }
                }
                EditorEffect::FocusTitle => {
                    self.title_state.update(cx, |s, cx| {
                        s.focus(window, cx);
                    });
                }
                EditorEffect::RequestSave => should_save = true,
            }
        }

        // Now that focus has moved off any removed blocks, drop their handles.
        self.block_input_states
            .retain(|h| note_blocks.iter().any(|b| b.id == h.block_id));

        // If focus died with a deleted block, land on the block now occupying
        // its neighborhood so the keyboard never dead-ends.
        let focused_valid = self
            .focused_block_id
            .as_ref()
            .map_or(true, |fid| note_blocks.iter().any(|b| b.id == *fid));
        if !focused_valid {
            if let Some(last) = note_blocks.last() {
                let id = last.id.clone();
                if let Some(handle) = self.block_input_states.iter().find(|h| h.block_id == id) {
                    let state = handle.state.clone();
                    state.update(cx, |s, cx| {
                        s.focus(window, cx);
                    });
                    self.focused_block_id = Some(id);
                }
            } else {
                self.focused_block_id = None;
            }
        }

        if should_save {
            if let Some(note) = self.selected_note() {
                let _ = self.storage.save(note);
            }
            self.schedule_save(cx);
        }
        cx.notify();
    }

    /// Move keyboard focus between blocks. Navigation is not a document
    /// mutation, so — like cascade-editor's view layer — it bypasses the
    /// reducer. Up/Down only escape the input at text boundaries.
    pub fn focus_adjacent_block(
        &mut self,
        is_up: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let block_id = match &self.focused_block_id {
            Some(id) => id.clone(),
            None => return,
        };

        let at_boundary = if let Some(handle) = self
            .block_input_states
            .iter()
            .find(|h| h.block_id == block_id)
        {
            let s = handle.state.read(cx);
            let cursor = s.cursor();
            let len = s.value().len();
            if is_up {
                cursor == 0
            } else {
                cursor >= len
            }
        } else {
            false
        };
        if !at_boundary {
            return; // let the input move the caret within its own text
        }

        let note = match self.selected_note() {
            Some(n) => n,
            None => return,
        };
        let idx = match note.blocks.iter().position(|b| b.id == block_id) {
            Some(i) => i,
            None => return,
        };

        let target_id = if is_up {
            if idx > 0 {
                Some(note.blocks[idx - 1].id.clone())
            } else {
                // Up from the very first block lands in the title
                self.title_state.update(cx, |s, cx| {
                    s.focus(window, cx);
                });
                self.focused_block_id = None;
                cx.notify();
                return;
            }
        } else if idx + 1 < note.blocks.len() {
            Some(note.blocks[idx + 1].id.clone())
        } else {
            None
        };

        if let Some(id) = target_id {
            if let Some(handle) = self.block_input_states.iter().find(|h| h.block_id == id) {
                let state = handle.state.clone();
                let land_at_end = !is_up;
                state.update(cx, |s, cx| {
                    s.focus(window, cx);
                    // Land at the sensible edge: start when moving up, end when moving down
                    let pos = if land_at_end { s.value().len() } else { 0 };
                    s.set_selected_range(pos..pos, cx);
                });
                self.focused_block_id = Some(id);
                cx.notify();
            }
        }
    }

    /// Slash-menu keyboard navigation from the canvas-level action handlers.
    /// Returns true when the key was consumed by the menu.
    pub fn slash_nav(&mut self, is_up: bool, cx: &mut Context<Self>) -> bool {
        let Some((_, query)) = self.slash_menu_target.clone() else {
            return false;
        };
        let filtered = crate::views::editor::slash_menu::filtered_slash_command_ids(&query);
        if filtered.is_empty() {
            return true; // consumed, but nothing to move through
        }
        let sel = self.slash_menu_selection.unwrap_or(0);
        self.slash_menu_selection = Some(if is_up {
            if sel == 0 {
                filtered.len() - 1
            } else {
                sel - 1
            }
        } else {
            (sel + 1) % filtered.len()
        });
        cx.notify();
        true
    }

    /// Closes the slash menu if open; returns true when it was.
    pub fn slash_close(&mut self, cx: &mut Context<Self>) -> bool {
        if self.slash_menu_target.is_some() {
            self.slash_menu_target = None;
            self.slash_menu_selection = None;
            cx.notify();
            true
        } else {
            false
        }
    }

    /// Fix 7: Schedules a debounced save and auto-clears the status badge after 2 seconds.
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

            // Fix 7: wait 2 more seconds then clear the status badge
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;

            let _ = this.update(cx, |app, cx| {
                app.status_message = None;
                cx.notify();
            });
        }));
    }
}
