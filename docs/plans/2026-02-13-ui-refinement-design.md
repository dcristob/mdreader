# UI Refinement Design: Toolbar, Typography, and Spacing

**Date:** 2026-02-13
**Status:** Approved

## Summary

Improve the mdreader UI in three areas: custom fonts for readability, better line spacing, and a refined floating toolbar.

## 1. Custom Fonts (Inter + JetBrains Mono)

- Bundle **Inter** (Regular + Bold) for body/headings and **JetBrains Mono** for code blocks
- Load via `ctx.add_font()` with `FontPriority::Highest` to replace egui defaults
- Place `.ttf` files in `fonts/` directory, included via `include_bytes!`
- Increase body text from 16px to 17px
- Fix render module's inconsistent 14px body default to match 17px
- ~400KB added to binary size

### Files changed
- `fonts/` — new directory with font files
- `src/theme.rs` — font loading logic
- `src/render.rs` — fix body text size to 17px

## 2. Line Height (1.35x)

- Set `TextFormat::line_height` to ~23px for body text (17px * 1.35)
- Scale proportionally for headings
- Increase `style.spacing.item_spacing.y` from ~4px to ~6px
- Increase heading spacing: 12px above, 6px below (up from 8px/4px)
- Only applies to render module (search view); CommonMarkViewer uses its own spacing but benefits from better font metrics

### Files changed
- `src/theme.rs` — global item_spacing adjustment
- `src/render.rs` — line_height on TextFormat, heading spacing

## 3. Toolbar Refinement

### Visual
- Remove `ui.group()` wrappers — no more nested borders
- Use `ui.separator()` between logical button groups (navigation | file ops | theme)
- Subtle drop shadow instead of stroke border
- Corner radius 10px (up from 8px)
- Tighter inner margins: horizontal 10px, vertical 8px

### Buttons
- Remove emoji prefixes — text-only labels
- Smaller button padding: 8px x 6px (down from 12px x 8px)
- Concise labels: "Back", "Fwd", "Open", "Dark"/"Light", "Search"

### Animation
- Keep current fade-in/fade-out behavior
- Reduce hover detection zone from 80px to 60px for snappier response

### Files changed
- `src/app.rs` — toolbar rendering overhaul, hover zone adjustment
