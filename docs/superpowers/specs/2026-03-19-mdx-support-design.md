# MDX File Support — Design Spec

## Goal

Allow mdreader to open, display, and navigate `.mdx` files with the same experience as `.md` files. MDX-specific syntax (JSX components) is handled by letting pulldown-cmark process them naturally. Import/export statements are stripped before rendering.

## Context

MDX files are markdown with embedded JSX, commonly used by documentation frameworks (Docusaurus, Nextra). The user's MDX files are mostly standard markdown with occasional JSX components. mdreader is a native Rust/egui app with no JS runtime, so JSX components cannot be executed.

## Approach: Minimal Extension + Line-Based Stripping

No new dependencies. Four focused changes:

### 1. Pre-processing: Strip import/export lines (src/file.rs)

- Add a function `strip_mdx_imports(content: &str) -> String` that removes lines starting with `import ` or `export ` (with optional leading whitespace)
- Applied only when the loaded file has a `.mdx` extension
- The stripping must be applied at **all four call sites** where content is set from a file:
  1. `app.rs` `load_file()` (line 118) — after `file::load_file(&path)` returns
  2. `app.rs` `load_file_without_history()` (line 137) — after `file::load_file(&path)` returns
  3. `app.rs` file watcher reload (line 180) — after `file::load_file(path)` returns
  4. `app.rs` drag-and-drop (line 171) — uses `self.load_file(path)`, so covered by call site 1
- The extension check (`path.extension() == Some("mdx")`) happens at each call site using `self.current_file`

### 2. File dialog filter (src/app.rs)

- Add `"mdx"` to the existing `add_filter("Markdown", &["md", "markdown"])` call, making it `add_filter("Markdown", &["md", "markdown", "mdx"])`

### 3. Link resolution (src/render.rs)

- Add `.mdx` to the existing `ends_with` checks so relative `.mdx` links are treated as internal file navigation (not opened in the browser)
- Note: this only applies to the search-active rendering path (`render_highlighted_markdown`). The non-search path uses `CommonMarkViewer::new().show()` which handles links internally via egui_commonmark — `.mdx` links there will fall through to the browser via the `OpenUrl` action, which is acceptable for now

### 4. Desktop entry

- Add `text/x-mdx` MIME type to `~/.local/share/applications/mdreader.desktop`

### Already working (no changes needed)

- **Drag-and-drop**: no extension filtering, accepts any file — `.mdx` works already
- **CLI argument**: no extension validation — `.mdx` works already
- **File watcher**: watches any file type — `.mdx` works already

## What is explicitly out of scope

- JSX tag stripping — pulldown-cmark handles HTML-like tags naturally (tags are dropped, inner text shows through)
- Multiline import handling — real-world MDX imports are single-line
- No new dependencies (no regex crate)
- No MDX-aware parser
- Non-search link handling for `.mdx` — acceptable to open in browser for now

## Trade-offs

- **Pro:** Minimal code changes (~15 lines), no new deps, easy to maintain
- **Con:** Rare edge cases (JSX expressions like `{props.foo}`) may appear as text; multiline imports won't be stripped; prose lines starting with `import ` or `export ` would be incorrectly stripped (acceptable given MDX docs content)
- **Upgrade path:** Can add regex-based or tag-stripping pre-processing later if needed
