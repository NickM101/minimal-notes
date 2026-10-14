# Notty

A clean, minimalist, GPU-accelerated desktop note-taking application for Windows, built with **GPUI** (the Rust UI framework created by the Zed team) and **GPUI Kit** (`gpui-kit.com`).

Inspired by monastic simplicity and the tactile beauty of paper, Notty offers an uncluttered writing space with instant search, interactive checklists, voice memos, markdown tools, and true pitch-black dark mode.

---

## ✨ Features

- **Apple Notes Aesthetic**: Two-pane minimalist layout with a sleek notes sidebar and borderless editor canvas.
- **GPU Accelerated**: Built on top of GPUI, offering 120 FPS capable hardware-accelerated rendering and fluid text layout.
- **Local Persistence**: Notes are automatically saved to local JSON documents under `notes_data/` on your computer as you type.
- **Instant Search**: Real-time filtering across titles and note contents.
- **Timestamp Formatting**: Intelligent date displays ("1:15 PM" for today, "Oct 14" for this year, and full date timestamps inside the editor).
- **Two-way Reactive Sync**: Editing title or body updates sidebar note cards, snippets, and storage instantly.

---

## 🛠️ Project Structure

```
minimal-notes/
├── Cargo.toml          # Project manifest with gpui-kit and dependencies
├── notes_data/         # Local folder storing JSON notes
├── src/
│   ├── main.rs         # GPUI Application shell, layout, toolbar, and editor views
│   ├── models.rs       # Note data model, serialization, and date formatting
│   └── storage.rs      # Local file storage and note lifecycle manager
└── README.md
```

---

## 🚀 Running the Application

### Prerequisites

1. **Rust & Cargo** (v1.92+ installed via `rustup`):
   ```powershell
   rustc --version
   cargo --version
   ```
2. **CMake** (v3.30+ on PATH):
   ```powershell
   cmake --version
   ```
3. **Visual Studio 2022 C++ Build Tools**:
   - Workload: "Desktop development with C++"
   - Includes MSVC compiler/linker (`link.exe`) and Windows SDK.

### Build and Run

To compile and launch the application:

```powershell
cargo run
```

Or for optimized release mode:

```powershell
cargo run --release
```
