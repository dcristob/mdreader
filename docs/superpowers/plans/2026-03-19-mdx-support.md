# MDX File Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow mdreader to open, display, and navigate `.mdx` files, stripping import/export lines before rendering.

**Architecture:** Add a `strip_mdx_imports` function in `src/file.rs`, apply it at all content-loading call sites when the file has `.mdx` extension, and extend file filters and link resolution to recognize `.mdx`.

**Tech Stack:** Rust, no new dependencies.

**Spec:** `docs/superpowers/specs/2026-03-19-mdx-support-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/file.rs` | Modify | Add `strip_mdx_imports` function |
| `src/app.rs` | Modify | Apply stripping at all load sites, update file dialog filter |
| `src/render.rs` | Modify | Add `.mdx` to link resolution |
| `mdreader.desktop` | Create | Desktop entry with `.mdx` MIME type (new file in repo root) |

---

### Task 1: Add `strip_mdx_imports` function

**Files:**
- Modify: `src/file.rs:30-35`

- [ ] **Step 1: Write the test**

Add a test module to `src/file.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_mdx_imports_removes_import_lines() {
        let input = "import { Callout } from '@components'\nimport React from 'react'\n\n# Hello\n\nSome text";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Hello\n\nSome text");
    }

    #[test]
    fn test_strip_mdx_imports_removes_export_lines() {
        let input = "export const meta = { title: 'Test' }\n\n# Hello";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Hello");
    }

    #[test]
    fn test_strip_mdx_imports_preserves_normal_markdown() {
        let input = "# Hello\n\nSome text with **bold**\n\n- list item";
        let result = strip_mdx_imports(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_strip_mdx_imports_handles_leading_whitespace() {
        let input = "  import Something from 'somewhere'\n\n# Title";
        let result = strip_mdx_imports(input);
        assert_eq!(result, "\n# Title");
    }

    #[test]
    fn test_strip_mdx_imports_preserves_prose_with_import_word() {
        // "import" mid-sentence should NOT be stripped
        let input = "# Title\n\nYou can import data from the API.";
        let result = strip_mdx_imports(input);
        assert_eq!(result, input);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib strip_mdx`
Expected: FAIL — `strip_mdx_imports` not found

- [ ] **Step 3: Write the implementation**

Add to `src/file.rs` after the `load_file` function:

```rust
pub fn strip_mdx_imports(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("import ") && !trimmed.starts_with("export ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib strip_mdx`
Expected: All 5 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/file.rs
git commit -m "feat: add strip_mdx_imports function for MDX support"
```

---

### Task 2: Wire stripping into all content-loading call sites

**Files:**
- Modify: `src/app.rs:107-152` (load_file, load_file_without_history)
- Modify: `src/app.rs:176-184` (file watcher reload)

The stripping must be applied at 3 code locations. Drag-and-drop (call site 4 in the spec) calls `self.load_file()`, so it is already covered by call site 1.

- [ ] **Step 1: Add import and helper to check if file is MDX**

In `src/app.rs`, add `use std::path::Path;` to the imports at the top of the file.

Then add a free function before the `impl MdReaderApp` block:

```rust
fn is_mdx_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("mdx"))
        .unwrap_or(false)
}
```

- [ ] **Step 2: Update `load_file` (call site 1)**

In `src/app.rs:118-121`, change:

```rust
match file::load_file(&path) {
    Ok(content) => {
        self.content = Some(content);
```

To:

```rust
match file::load_file(&path) {
    Ok(content) => {
        self.content = Some(if is_mdx_file(&path) {
            file::strip_mdx_imports(&content)
        } else {
            content
        });
```

- [ ] **Step 3: Update `load_file_without_history` (call site 2)**

In `src/app.rs:137-140`, apply the same change:

```rust
match file::load_file(&path) {
    Ok(content) => {
        self.content = Some(if is_mdx_file(&path) {
            file::strip_mdx_imports(&content)
        } else {
            content
        });
```

- [ ] **Step 4: Update file watcher reload (call site 3)**

In `src/app.rs:179-182`, change:

```rust
if let Some(ref path) = self.current_file {
    if let Ok(content) = file::load_file(path) {
        self.content = Some(content);
    }
}
```

To:

```rust
if let Some(ref path) = self.current_file {
    if let Ok(content) = file::load_file(path) {
        self.content = Some(if is_mdx_file(path) {
            file::strip_mdx_imports(&content)
        } else {
            content
        });
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

- [ ] **Step 6: Commit**

```bash
git add src/app.rs
git commit -m "feat: apply MDX import stripping at all content-loading sites"
```

---

### Task 3: Update file dialog filter and link resolution

**Files:**
- Modify: `src/app.rs:358` (file dialog filter)
- Modify: `src/render.rs:225` (link resolution)

- [ ] **Step 1: Update file dialog filter**

In `src/app.rs:358`, change:

```rust
.add_filter("Markdown", &["md", "markdown"])
```

To:

```rust
.add_filter("Markdown", &["md", "markdown", "mdx"])
```

- [ ] **Step 2: Update link resolution**

In `src/render.rs:225`, change:

```rust
if clean_url.ends_with(".md") || clean_url.ends_with(".markdown") {
```

To:

```rust
if clean_url.ends_with(".md") || clean_url.ends_with(".markdown") || clean_url.ends_with(".mdx") {
```

> **Note:** This only affects the search-active rendering path (`render_highlighted_markdown`). The non-search path uses `CommonMarkViewer::new().show()` which handles links internally — `.mdx` links there will open in the browser. This is an accepted limitation per the spec.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/render.rs
git commit -m "feat: recognize .mdx in file dialog and link navigation"
```

---

### Task 4: Create desktop entry and build

**Files:**
- Create: `mdreader.desktop` (repo root, for distribution)

- [ ] **Step 1: Create desktop entry file**

Create `mdreader.desktop` in the repo root:

```ini
[Desktop Entry]
Name=Markdown Reader
Comment=A fast, lightweight markdown viewer
Exec=mdreader %f
Icon=mdreader
Terminal=false
Type=Application
Categories=Utility;TextEditor;
MimeType=text/markdown;text/x-markdown;text/x-mdx;
```

- [ ] **Step 2: Build release and install**

Run:

```bash
cargo build --release && cp target/release/mdreader ~/.local/bin/mdreader
```

- [ ] **Step 3: Install desktop entry**

```bash
cp mdreader.desktop ~/.local/share/applications/mdreader.desktop
```

- [ ] **Step 4: Manual smoke test**

Open an `.mdx` file to verify:

```bash
~/.local/bin/mdreader some-file.mdx
```

Expected: file opens, import/export lines are not visible, markdown renders normally.

- [ ] **Step 5: Commit**

```bash
git add mdreader.desktop
git commit -m "feat: add desktop entry with MDX MIME type support"
```

---

### Task 5: Version bump

**Files:**
- Modify: `Cargo.toml:3`

- [ ] **Step 1: Bump version**

In `Cargo.toml:3`, change:

```toml
version = "0.2.0"
```

To:

```toml
version = "0.2.1"
```

- [ ] **Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.1 for MDX support"
```
