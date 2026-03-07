# mdreader

A desktop markdown file viewer built with Rust and egui.

## Project Structure

- `src/main.rs` — Entry point, window setup, icon loading
- `src/app.rs` — Main application state and UI (toolbar, search, panels)
- `src/render.rs` — Markdown rendering with syntax highlighting and search match overlay
- `src/markdown.rs` — Markdown parsing (link extraction via pulldown-cmark)
- `src/search.rs` — Text search with match tracking
- `src/file.rs` — File loading and file watcher (notify)
- `src/args.rs` — CLI argument parsing (accepts file path as first arg)
- `src/theme.rs` — Theme/styling configuration
- `src/config.rs` — User configuration
- `src/links.rs` — Link handling

## Key Dependencies

- `eframe`/`egui` 0.33 — GUI framework
- `pulldown-cmark` — Markdown parsing
- `syntect` — Syntax highlighting
- `notify` — File change watching
- `rfd` — File dialog

## Installation

The binary is installed system-wide at `~/.local/bin/mdreader` with a desktop entry for double-click `.md` file opening.

**After any code modification, rebuild and reinstall:**

```sh
cargo build --release && cp target/release/mdreader ~/.local/bin/mdreader
```

## Build & Install Rule

**Any change to this project must include updating the installed binary.** After modifying code, always run:

```sh
cargo build --release && cp target/release/mdreader ~/.local/bin/mdreader
```

## Version Bump Rule

**Every push to `main` must include a version bump in `Cargo.toml`.** The GitHub Actions workflow creates a release tagged with the version from `Cargo.toml`, so pushing the same version twice will fail. Bump the patch version (e.g., `0.1.0` -> `0.1.1`) for small changes, minor for features, major for breaking changes.

## Desktop Integration

- Desktop entry: `~/.local/share/applications/mdreader.desktop`
- Icon: `~/.local/share/icons/mdreader.png`
- Default handler for: `text/markdown`, `text/x-markdown`
