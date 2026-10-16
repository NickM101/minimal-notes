use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chrono::{Local, TimeZone};
use rusqlite::Connection;

use crate::models::Note;

/// Local persistence backed by a single SQLite database file.
///
/// Each note is one row: searchable columns (`title`, timestamps) plus the
/// full serialized note (`data` JSON) so structured blocks round-trip without
/// a relational explosion. Writes are upserts inside a transaction; reads are
/// a single query ordered by `updated_at`.
///
/// On first run against an empty database, notes from the legacy JSON file
/// store (`notes_data/*.json`, relative to the working directory of earlier
/// versions) are imported once. Legacy files are left in place untouched.
#[derive(Clone)]
pub struct Storage {
    db_path: PathBuf, // exposed via `path()` for diagnostics
    conn: Arc<Mutex<Connection>>,
}

/// Platform-appropriate data directory: `%APPDATA%\minimal-notes` on Windows,
/// `$XDG_DATA_HOME/minimal-notes` (or `~/.local/share/minimal-notes`) on
/// Linux and the other Unixes.
fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("minimal-notes")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .unwrap_or_else(|| PathBuf::from("."))
            .join("minimal-notes")
    }
}

impl Storage {
    /// Opens (creating if needed) the database in the platform data directory.
    /// Also imports the legacy JSON store (`notes_data/` relative to the
    /// working directory) on first run, matching where earlier versions saved.
    ///
    /// Falls back to a `notes_data` directory beside the working directory
    /// when the platform data location is not writable, so a read-only
    /// HOME never blocks the app from starting.
    pub fn new() -> Self {
        let primary = default_data_dir();
        Self::open_inner(&primary, Some(PathBuf::from("notes_data"))).unwrap_or_else(|err| {
            eprintln!(
                "Could not open notes database at {} ({err}); falling back to ./notes_data",
                primary.display()
            );
            Self::open_inner(&PathBuf::from("notes_data"), None)
                .expect("failed to open the notes database in the working directory")
        })
    }

    /// Opens (creating if needed) the database inside `dir`. Legacy JSON is
    /// only imported from `dir/notes_data`, so tests never touch a real store.
    #[allow(dead_code)] // public API; production startup goes through `new`
    pub fn open(dir: &std::path::Path) -> std::io::Result<Self> {
        Self::open_inner(dir, None)
    }

    fn open_inner(
        dir: &std::path::Path,
        extra_legacy_dir: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        fs::create_dir_all(dir)?;
        let db_path = dir.join("notes.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // WAL keeps readers and the periodic writer from blocking each other;
        // NORMAL is the standard durable-enough checkpoint mode for it.
        conn.pragma_update(None, "journal_mode", "WAL")
            .and_then(|_| conn.pragma_update(None, "synchronous", "NORMAL"))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                id         TEXT PRIMARY KEY,
                title      TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                data       TEXT NOT NULL
            );",
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let storage = Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.import_legacy_json(dir, extra_legacy_dir.as_deref())?;

        Ok(storage)
    }

    /// One-time import of the pre-SQLite JSON store, only when the database
    /// holds no notes yet (so it never resurrects deleted notes afterwards).
    fn import_legacy_json(
        &self,
        dir: &std::path::Path,
        extra_legacy_dir: Option<&std::path::Path>,
    ) -> std::io::Result<()> {
        let candidates = [dir.join("notes_data")];
        let legacy_dir = candidates.into_iter().find(|d| d.is_dir()).or_else(|| {
            extra_legacy_dir
                .filter(|d| d.is_dir())
                .map(Path::to_path_buf)
        });
        let Some(legacy_dir) = legacy_dir else {
            return Ok(());
        };

        let count: i64 = {
            let conn = self.lock_conn();
            conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        };
        if count > 0 {
            return Ok(());
        }

        let mut imported = 0usize;
        if let Ok(entries) = fs::read_dir(&legacy_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mut note) = serde_json::from_str::<Note>(&content) {
                        note.migrate_if_needed();
                        if self.save(&note).is_ok() {
                            imported += 1;
                        }
                    }
                }
            }
        }
        if imported > 0 {
            eprintln!(
                "Imported {imported} note(s) from legacy JSON store at {}",
                legacy_dir.display()
            );
        }
        Ok(())
    }

    /// Loads every note, newest first.
    pub fn load_all(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        let conn = self.lock_conn();
        let Ok(mut stmt) = conn.prepare("SELECT data FROM notes ORDER BY updated_at DESC") else {
            return notes;
        };
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                if let Ok(mut note) = serde_json::from_str::<Note>(&row) {
                    note.migrate_if_needed();
                    notes.push(note);
                }
            }
        }
        notes
    }

    /// Inserts or updates a note (atomic upsert keyed by id).
    pub fn save(&self, note: &Note) -> std::io::Result<()> {
        let json = serde_json::to_string(note)?;
        let conn = self.lock_conn();
        conn.execute(
            "INSERT INTO notes (id, title, created_at, updated_at, data)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                data = excluded.data",
            rusqlite::params![
                note.id,
                note.title,
                note.created_at.timestamp_millis(),
                note.updated_at.timestamp_millis(),
                json,
            ],
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    /// Deletes a note; deleting an unknown id is a no-op.
    pub fn delete(&self, id: &str) -> std::io::Result<()> {
        let conn = self.lock_conn();
        conn.execute("DELETE FROM notes WHERE id = ?1", [id])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    /// Number of stored notes (used by tests and diagnostics).
    #[allow(dead_code)] // consumed by the integration tests
    pub fn count(&self) -> std::io::Result<i64> {
        let conn = self.lock_conn();
        conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    #[allow(dead_code)] // diagnostics helper
    pub fn path(&self) -> &std::path::Path {
        &self.db_path
    }

    fn lock_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a note's timestamps back out of the millisecond columns.
#[allow(dead_code)]
fn millis_to_local(ms: i64) -> Option<chrono::DateTime<Local>> {
    Local.timestamp_millis_opt(ms).single()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Block, BlockKind};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("notty-test-{}-{}", name, uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn empty_database_starts_empty() {
        let storage = Storage::open(&temp_dir("empty")).unwrap();
        assert_eq!(storage.count().unwrap(), 0);
        assert!(storage.load_all().is_empty(), "no seed data expected");
    }

    #[test]
    fn save_then_load_roundtrips_blocks() {
        let storage = Storage::open(&temp_dir("roundtrip")).unwrap();
        let mut note = Note::new("", "");
        note.title = "Groceries".into();
        note.blocks = vec![
            Block::new(BlockKind::Paragraph {
                text: "milk".into(),
            }),
            Block::new(BlockKind::Code {
                language: "rust".into(),
                text: "fn main() {}".into(),
            }),
        ];
        storage.save(&note).unwrap();

        let loaded = storage.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, note.id);
        assert_eq!(loaded[0].title, "Groceries");
        assert_eq!(loaded[0].blocks.len(), 2);
        assert_eq!(loaded[0].blocks[0].text(), "milk");
        assert!(matches!(
            &loaded[0].blocks[1].kind,
            BlockKind::Code { text, .. } if text == "fn main() {}"
        ));
    }

    /// A note loaded from storage must survive being saved again and loaded
    /// once more — the save path of a deserialized note is exactly where the
    /// old serde flatten bug corrupted blocks into unloadable JSON.
    #[test]
    fn loaded_note_survives_resave() {
        let dir = temp_dir("resave");
        let storage = Storage::open(&dir).unwrap();
        let mut note = Note::new("Re-save", "");
        note.blocks = vec![Block::new(BlockKind::Quote { text: "q".into() })];
        storage.save(&note).unwrap();

        let loaded = &storage.load_all()[0];
        storage.save(loaded).unwrap();
        let reopened = Storage::open(&dir).unwrap().load_all();
        assert_eq!(reopened.len(), 1);
        assert!(matches!(&reopened[0].blocks[0].kind, BlockKind::Quote { text } if text == "q"));
    }

    #[test]
    fn save_upserts_updates_in_place() {
        let storage = Storage::open(&temp_dir("upsert")).unwrap();
        let mut note = Note::new("Draft", "v1");
        storage.save(&note).unwrap();

        note.title = "Renamed".into();
        note.updated_at += chrono::Duration::seconds(5);
        note.blocks[0].set_text("v2");
        storage.save(&note).unwrap();

        assert_eq!(
            storage.count().unwrap(),
            1,
            "upsert must not duplicate rows"
        );
        let loaded = storage.load_all();
        assert_eq!(loaded[0].title, "Renamed");
        assert_eq!(loaded[0].blocks[0].text(), "v2");
    }

    #[test]
    fn load_all_sorts_newest_first() {
        let storage = Storage::open(&temp_dir("ordering")).unwrap();
        let mut older = Note::new("Older", "a");
        older.updated_at -= chrono::Duration::hours(2);
        let mut newer = Note::new("Newer", "b");
        newer.updated_at -= chrono::Duration::hours(1);
        storage.save(&older).unwrap();
        storage.save(&newer).unwrap();

        let loaded = storage.load_all();
        let titles: Vec<&str> = loaded.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, vec!["Newer", "Older"]);
    }

    #[test]
    fn delete_removes_only_target_note() {
        let storage = Storage::open(&temp_dir("delete")).unwrap();
        let keep = Note::new("Keep", "k");
        let drop = Note::new("Drop", "d");
        storage.save(&keep).unwrap();
        storage.save(&drop).unwrap();

        storage.delete(&drop.id).unwrap();
        storage.delete("nonexistent").unwrap(); // no-op, no error

        let loaded = storage.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, keep.id);
    }

    #[test]
    fn data_persists_across_reopens() {
        let dir = temp_dir("persist");
        let note = Note::new("Durable", "survives reopen");
        Storage::open(&dir).unwrap().save(&note).unwrap();

        let reopened = Storage::open(&dir).unwrap();
        let loaded = reopened.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Durable");
    }

    #[test]
    fn legacy_json_notes_are_imported_once() {
        let dir = temp_dir("legacy");
        let legacy = dir.join("notes_data");
        fs::create_dir_all(&legacy).unwrap();
        let note = Note::new("From JSON", "legacy body");
        fs::write(
            legacy.join(format!("{}.json", note.id)),
            serde_json::to_string_pretty(&note).unwrap(),
        )
        .unwrap();

        let storage = Storage::open(&dir).unwrap();
        assert_eq!(storage.count().unwrap(), 1);
        assert_eq!(storage.load_all()[0].title, "From JSON");

        // A second open must not re-import (and definitely must not
        // resurrect anything deleted in between).
        let note2 = Note::new("New note", "sqlite native");
        storage.save(&note2).unwrap();
        storage.delete(&storage.load_all()[1].id).unwrap(); // delete imported one

        let reopened = Storage::open(&dir).unwrap();
        assert_eq!(reopened.count().unwrap(), 1);
        assert_eq!(reopened.load_all()[0].title, "New note");
    }
}
