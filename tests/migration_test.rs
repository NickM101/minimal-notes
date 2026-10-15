use minimal_notes::models::{BlockKind, Note};

#[test]
fn test_legacy_note_migration_fixture() {
    let legacy_fixture = r#"{
  "id": "759adae7-575a-4184-be9b-1f924c36218e",
  "title": "Minimalism",
  "body": "First I admit that I am not a web designer, but a design lover.\n\nMinimalism refers to a certain style and applies in many fields: Art, architecture, literature, and philosophy as a ==way of life==.\n\n> [!NOTE] Zen is not the lack of clutter; it is the deliberate presence of purpose.\n\n## Core Principles\n- [x] Strip work down to its most **fundamental features**\n- [x] Embrace deliberate whitespace and calm typography\n- [ ] Focus on tactile, physical book aesthetic\n- [ ] Practice daily reflection and minimal journaling\n\n> Simplicity is the ultimate sophistication.\n\n```rust\nfn zen_state() -> &'static str {\n    \"Serenity in simplicity\"\n}\n```\n\nAre you a designer who hides behind ornament, or one who crafts `essential harmony`?",
  "checklist": [
    {
      "id": "chk-1",
      "text": "I have read about that topic.",
      "checked": false
    },
    {
      "id": "chk-2",
      "text": "I have read about that topic.",
      "checked": true
    }
  ],
  "images": [],
  "voice": {
    "id": "rec-1",
    "duration_seconds": 64,
    "current_seconds": 18,
    "is_playing": false
  },
  "created_at": "2026-10-14T18:59:21.080818900+03:00",
  "updated_at": "2026-10-14T18:59:21.080818900+03:00"
}"#;

    let mut note: Note = serde_json::from_str(legacy_fixture).expect("Failed to deserialize legacy note");
    assert!(note.blocks.is_empty());
    assert_eq!(note.checklist.len(), 2);
    assert!(note.voice.is_some());

    note.migrate_if_needed();

    assert!(!note.blocks.is_empty(), "Blocks should be populated after migration");
    assert_eq!(note.schema_version, 1);
    assert!(note.body.is_none());
    assert!(note.checklist.is_empty());

    // Check that heading was parsed
    let has_heading = note.blocks.iter().any(|b| matches!(&b.kind, BlockKind::Heading { level: 2, text } if text == "Core Principles"));
    assert!(has_heading, "Heading block should be extracted");

    // Check that code block was parsed
    let has_code = note.blocks.iter().any(|b| matches!(&b.kind, BlockKind::Code { language, .. } if language == "rust"));
    assert!(has_code, "Code block should be extracted with language");

    // Check that checklist items were migrated as Todo blocks appended
    let todo_count = note.blocks.iter().filter(|b| matches!(&b.kind, BlockKind::Todo { .. })).count();
    // 4 from body markdown + 2 from legacy checklist
    assert_eq!(todo_count, 6, "Expected 4 body checkboxes + 2 legacy checklist items = 6 todos");

    // Check that voice recording was kept unchanged
    assert_eq!(note.voice.as_ref().unwrap().id, "rec-1");

    // Round-trip serialization check
    let serialized = serde_json::to_string(&note).expect("Serialization failed");
    let round_trip: Note = serde_json::from_str(&serialized).expect("Round-trip deserialization failed");
    assert_eq!(round_trip.blocks.len(), note.blocks.len());
    assert_eq!(round_trip.schema_version, 1);
}

#[test]
fn test_unknown_block_round_trip() {
    let json_with_unknown = r#"{
  "id": "unknown-test",
  "schema_version": 1,
  "title": "Future Note",
  "blocks": [
    {
      "id": "b-1",
      "type": "future_ai_block",
      "model": "gemini-future",
      "data": 42
    },
    {
      "id": "b-2",
      "type": "paragraph",
      "text": "Standard paragraph"
    }
  ],
  "created_at": "2026-10-14T18:59:21.080818900+03:00",
  "updated_at": "2026-10-14T18:59:21.080818900+03:00"
}"#;

    let note: Note = serde_json::from_str(json_with_unknown).expect("Failed to deserialize unknown block note");
    assert_eq!(note.blocks.len(), 2);
    assert_eq!(note.blocks[0].kind, BlockKind::Unknown);
    // Check that extra fields are preserved in HashMap
    assert_eq!(note.blocks[0].extra.get("model").unwrap(), &serde_json::json!("gemini-future"));
    assert_eq!(note.blocks[0].extra.get("data").unwrap(), &serde_json::json!(42));

    // Serialize and verify round-trip preserves extra fields
    let reserialized = serde_json::to_string(&note).expect("Failed to re-serialize");
    assert!(reserialized.contains("future_ai_block"));
    assert!(reserialized.contains("gemini-future"));
}
