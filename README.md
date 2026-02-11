# Markdown Viewer

A fast, lightweight native Linux markdown viewer built with Rust and egui.

## Features

- **Fast Rendering** - Immediate mode GUI with egui for smooth performance
- **CLI Support** - Open files from command line: `mdreader file.md`
- **File Picker** - GUI file dialog when no CLI argument provided
- **Full Markdown Support** - Headers, lists, code blocks, links, tables, blockquotes
- **Syntax Highlighting** - Code blocks with language detection
- **Clickable Links** - External URLs open in browser, internal links navigate
- **Search** - Ctrl+F to search within document
- **Themes** - Dark/light mode toggle
- **Zoom** - Adjustable font size (50% - 300%)
- **Auto-Reload** - Watches files and reloads on external changes
- **Navigation History** - Back/forward buttons for internal links
- **Status Bar** - Shows filename, line count, character count

## Installation

```bash
cargo build --release
```

The binary will be at `./target/release/mdreader`

## Usage

```bash
# Open specific file
mdreader document.md

# Open without file (shows file picker)
mdreader
```

## Keyboard Shortcuts

- `Ctrl+F` - Toggle search bar
- `Back/Forward buttons` - Navigate history

## Tech Stack

- Rust
- egui + eframe - Immediate mode GUI
- pulldown-cmark - Markdown parsing
- syntect - Syntax highlighting
- egui_commonmark - Markdown rendering
- notify - File watching
- rfd - File dialogs
- webbrowser - Opening external links

## License

MIT
