# PasteGoblin

A fast desktop meme picker for Windows. Browse your meme library, search by name, and copy images straight to your clipboard for pasting into Discord, Messenger, Slack, or anywhere else.

Built with [Tauri v2](https://tauri.app/), React 19, and Rust.

## Features

- **Instant clipboard copy** — select a meme and hit `Ctrl+C` or `Enter` to copy it as an image to your clipboard
- **Global hotkey** — `Ctrl+Shift+M` toggles the window from anywhere
- **Fuzzy search** — quickly find memes by name
- **Recently used** — your most recently copied memes appear at the top
- **Drag & drop upload** — drag images into the upload modal or click to browse
- **Image replacement** — swap out a meme's image from the edit modal
- **Duplicate detection** — SHA-256 hashing prevents importing the same file twice
- **Paginated list** — handles large libraries with page navigation
- **System tray** — runs in the background with a tray icon and toggle/quit menu
- **Dark theme** — dark purple background with lime green accents

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Shift+M` | Toggle window (global) |
| `Arrow Up/Down` | Navigate meme list |
| `Enter` | Copy selected meme to clipboard |
| `Ctrl+C` | Copy selected meme to clipboard |
| `Ctrl+S` | Download/save selected meme |
| `Ctrl+Delete` | Delete selected meme |
| `Esc` | Close modal or hide window |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Frontend | React 19, TypeScript, Vite |
| Backend | Rust |
| Database | SQLite (rusqlite) |
| Clipboard | arboard + image crate |
| Search | fuse.js |
| Icons | lucide-react |
| File picker | tauri-plugin-dialog |
| File access | tauri-plugin-fs |
| Hotkey | tauri-plugin-global-shortcut |

## Project Structure

```
src/                          # React frontend
  app/                        # App entry point
  features/
    layout/                   # MainWindow, Header, StatusBar
    meme-list/                # Paginated meme list with recently used
    meme-detail/              # Detail panel with preview and actions
    upload/                   # Upload/edit modal with drag & drop
  shared/                     # MemeImage component, file-url helper, types

src-tauri/                    # Rust backend
  src/
    lib.rs                    # Tauri commands and app setup
    db.rs                     # SQLite database layer
    models.rs                 # Data models (Meme, Category)
    main.rs                   # Entry point
  capabilities/               # Tauri permission config
  icons/                      # Platform app icons
```

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri v2 system dependencies — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Getting Started

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

The production build outputs to `src-tauri/target/release/bundle/`.

## Data Storage

Memes and the SQLite database are stored in your system's app data directory:

```
%APPDATA%/com.codystine.paste-goblin/
  db.sqlite          # Meme metadata
  memes/             # Imported image files (renamed to UUID)
```

## License

MIT
