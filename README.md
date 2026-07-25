# Charly — The connected notes app

**An infinite corkboard for your thoughts, files, and screenshots — with visual connections between ideas.**

Built with Tauri + React + Rust. *Vibe coded in a weekend with Claude Code.*

![License](https://img.shields.io/badge/license-AGPL--3.0-blue)

---

## What is this?

Charly watches your system clipboard. Copy anything — text, images, files — and it appears as a sticky note on an infinite canvas. Move notes around, resize them, connect them with colored strings. It's a spatial memory aid that works the way your brain does: visually, associatively, non-linearly.

### Features

| Feature | Description |
|---|---|
| **Clipboard watch** | Copy text, screenshots, or files — they auto-appear as notes |
| **Infinite canvas** | Pan (alt+drag), zoom (ctrl+scroll), place notes anywhere |
| **Sticky notes** | Drag, resize, 8 color presets, type directly into text notes |
| **File notes** | Click to open in file manager, copy path back to clipboard |
| **String connections** | Pin notes together with colored lines — build mind maps |
| **System tray** | Lives in your tray, close to hide, always watching |
| **Themes** | Light (corkboard), dark, high-contrast modes |
| **Persistence** | SQLite-backed — all notes and connections survive restarts |
| **Cross-platform** | Linux, macOS, Windows (via Tauri) |

### Architecture

```
┌─────────────────────────────────┐
│  Tauri Desktop Shell             │
│                                  │
│  ┌───────────┐  ┌─────────────┐ │
│  │ React     │  │ Rust Backend │ │
│  │ Frontend  │◄─┤              │ │
│  │ (canvas,  │  │ • Clipboard  │ │
│  │  notes,   │  │   watcher    │ │
│  │  lines)   │  │ • SQLite DB  │ │
│  │           │  │ • System     │ │
│  │           │  │   tray       │ │
│  └───────────┘  └─────────────┘ │
└─────────────────────────────────┘
```

| Layer | Stack |
|---|---|
| Desktop shell | [Tauri v2](https://tauri.app/) |
| Canvas UI | React + TypeScript |
| Clipboard | [arboard](https://crates.io/crates/arboard) (Rust) |
| Storage | [rusqlite](https://crates.io/crates/rusqlite) (SQLite) |
| File opening | [open](https://crates.io/crates/open) (xdg-open) |

## Getting started

### Prerequisites

- [Rust](https://rustup.rs) (1.77+)
- [Node.js](https://nodejs.org) (24+)
- System dependencies for Tauri on Linux:

```bash
sudo apt install -y pkg-config libglib2.0-dev libgtk-3-dev \
  libsoup-3.0-dev libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev
```

### Dev mode

```bash
git clone https://github.com/chukrobertson/charly.git
cd charly
npm install
npx tauri dev
```

### Build

```bash
npx tauri build
# Produces .deb, .rpm, and .AppImage in src-tauri/target/release/bundle/
```

## Usage

- **Copy** text, images, or files anywhere on your system — they'll appear as notes
- **Ctrl+Scroll** to zoom the canvas
- **Alt+Drag** or middle-click to pan
- **Drag** notes to reposition, **bottom-right corner** to resize
- **Double-click** a note to cycle through colors
- **Click the star** on one note, then on another — they connect with a colored line
- **Click the sun icon** to toggle light/dark/contrast themes
- **Close the window** — it hides to tray and keeps running

## Data storage

All data lives on your machine:

```
~/.local/share/com.charly.app/
  ├── cliffnote.db     # SQLite (notes + connections)
  └── blobs/           # Copied images and files
```

## License

GNU Affero General Public License v3.0 — see [LICENSE](LICENSE).

## Credits

This project was vibe coded as a collaboration between human intent and [Claude Code](https://claude.ai). The entire codebase was generated through iterative prompting — describing features in natural language and letting AI produce the implementation. No manual code was written.

The name "Charly" is a playful riff on "Charlie" — a faithful companion that remembers things for you.
