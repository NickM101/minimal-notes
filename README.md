# Notty

A clean, minimalist, GPU-accelerated desktop note-taking app for **Linux and
Windows**, built with [GPUI](https://gpui-kit.com) (the Rust UI framework from
the Zed team) and **GPUI Kit**.

Inspired by monastic simplicity and the tactile beauty of paper, Notty offers
an uncluttered writing space with a Notion-style block editor, instant search,
interactive checklists, and true pitch-black dark mode — with every note
stored locally in **SQLite**.

---

## ✨ Features

- **Block editor canvas** — paragraphs, headings, bullets, numbered lists,
  to-dos, quotes, code snippets and dividers, with `/` slash commands and
  markdown shortcuts (`# `, `- `, `> `, ` ``` `, `---`).
- **SQLite persistence** — notes live in a single database file
  (`notes.db`) in your platform data directory, written through WAL-mode
  SQLite with atomic upserts. Auto-saving as you type.
- **Automatic migration** — first launch imports notes from older
  JSON-file versions of the app (`notes_data/*.json`); your data is never
  lost across upgrades.
- **Instant search** — real-time filtering across titles and contents.
- **List & grid browse layouts**, zen focus mode, light/dark theme.
- **GPU accelerated** — 120 FPS-capable hardware rendering via GPUI.
- **Self-contained binary** — icons and artwork are embedded at compile
  time; the executable never depends on files next to it.

## 📦 Storage location

| Platform | Path |
| --- | --- |
| Linux | `$XDG_DATA_HOME/minimal-notes/notes.db` (default `~/.local/share/minimal-notes/`) |
| Windows | `%APPDATA%\minimal-notes\notes.db` |

If the platform directory is not writable, Notty falls back to a local
`notes_data/` folder and keeps working.

---

## 🚀 Download

Prebuilt binaries are attached to each
[GitHub release](../../releases):

- `notty-vX.Y.Z-x86_64-linux.tar.gz`
- `notty-vX.Y.Z-x86_64-windows.zip`

Unpack and run the `notty` (Linux) or `Notty.exe` (Windows) executable —
no installer or runtime dependencies required.

## 🛠️ Building from source

### Prerequisites

- **Rust** 1.85+ via [rustup](https://rustup.rs)
- **Linux**: standard C toolchain plus the libraries GPUI needs
  (`libclang`/`clang`, `libxkbcommon`, and a Vulkan or EGL-capable GPU stack)
  - Debian/Ubuntu: `sudo apt install build-essential clang libxkbcommon-dev`
- **Windows**: Visual Studio 2022 **C++ Build Tools**
  (workload *Desktop development with C++*) and CMake on PATH

### Build and run

```bash
cargo run            # development build
cargo run --release  # optimized build
```

### Tests

The project ships unit and integration tests covering the document reducer,
serde round-trip integrity of blocks, and full CRUD against the SQLite store
(including legacy-JSON migration and cross-reopen durability):

```bash
cargo test
```

---

## 🗂️ Project structure

```
minimal-notes/
├── src/
│   ├── main.rs               # App shell, window setup, embedded assets
│   ├── storage.rs            # SQLite persistence + legacy JSON migration
│   ├── models/               # Note & Block models with schema versioning
│   ├── markdown.rs           # Markdown → block parsing
│   ├── app/                  # Editor state: reducer, actions, input handles
│   │   └── editor_reducer.rs # Pure document reducer (unit-tested)
│   ├── views/                # Editor canvas, slash menu, browse screens
│   ├── components/           # Title bar, branding, modals
│   └── platform/             # Platform integration (Win32 window focus)
├── tests/                    # Integration tests (migration, storage CRUD)
└── .github/workflows/        # Release CI: Linux + Windows binaries on tag
```

## 🏗️ Architecture notes

Document mutations flow through a small unidirectional pipeline: keyboard and
pointer events emit `EditorAction`s, a **pure reducer** (`app/editor_reducer.rs`)
turns them into new block vectors plus effects (focus, save), and the host
applies those effects to live input states. The reducer has zero UI
dependencies, so document semantics are unit-tested without a window.

## 📄 License

[MIT](LICENSE)
