# MDX File Support — Design Spec

## Goal

Allow mdreader to open, display, and navigate `.mdx` files with the same experience as `.md` files. MDX-specific syntax (JSX components) is handled by letting pulldown-cmark process them naturally. Import/export statements are stripped before rendering.

## Context

MDX files are markdown with embedded JSX, commonly used by documentation frameworks (Docusaurus, Nextra). The user's MDX files are mostly standard markdown with occasional JSX components. mdreader is a native Rust/egui app with no JS runtime, so JSX components cannot be executed.

## Approach: Minimal Extension + Line-Based Stripping

No new dependencies. Four focused changes:

### 1. Pre-processing: Strip import/export lines

- Add a function (e.g., `strip_mdx_imports`) that removes lines starting with `import ` or `export ` (with optional leading whitespace)
- Applied only when the loaded file has a `.mdx` extension
- Runs before content is passed to pulldown-cmark

### 2. File dialog filter (src/app.rs)

- Add `"mdx"` to the existing markdown file filter: `["md", "markdown", "mdx"]`

### 3. Link resolution (src/render.rs)

- Add `.mdx` to the existing `ends_with` checks so relative `.mdx` links are treated as internal file navigation (not opened in the browser)

### 4. Desktop entry

- Add `text/mdx` MIME type to `~/.local/share/applications/mdreader.desktop`

## What is explicitly out of scope

- JSX tag stripping — pulldown-cmark handles HTML-like tags naturally (tags are dropped, inner text shows through)
- Multiline import handling — real-world MDX imports are single-line
- No new dependencies (no regex crate)
- No MDX-aware parser

## Trade-offs

- **Pro:** Minimal code changes (~15 lines), no new deps, easy to maintain
- **Con:** Rare edge cases (JSX expressions like `{props.foo}`) may appear as text; multiline imports won't be stripped
- **Upgrade path:** Can add regex-based or tag-stripping pre-processing later if needed
