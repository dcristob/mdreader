# UI Refinement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve mdreader's readability and toolbar aesthetics with custom fonts (Inter + JetBrains Mono), better line spacing (1.35x), and a refined floating toolbar.

**Architecture:** Three independent changes to the egui-based markdown viewer. Font loading happens once at startup in `theme.rs`. Line height is set per `TextFormat` in `render.rs`. Toolbar is restyled in `app.rs` by removing grouped borders, dropping emojis, and adding a shadow.

**Tech Stack:** Rust, egui 0.33, eframe 0.33, egui_commonmark 0.22

---

### Task 1: Download and add font files

**Files:**
- Create: `fonts/Inter-Regular.ttf`
- Create: `fonts/Inter-Bold.ttf`
- Create: `fonts/JetBrainsMono-Regular.ttf`

**Step 1: Create fonts directory**

```bash
mkdir -p fonts/
```

**Step 2: Download Inter font files**

Download Inter Regular and Bold `.ttf` files from the official GitHub releases (https://github.com/rsms/inter/releases). Place them in `fonts/`.

```bash
# Download Inter from GitHub releases
cd fonts/
curl -L -o inter.zip "https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip"
unzip inter.zip "Inter-4.1/InterVariable.ttf" "Inter-4.1/InterVariable-Italic.ttf" 2>/dev/null || true
# We need the static TTF versions
unzip -j inter.zip "Inter-4.1/extras/ttf/Inter-Regular.ttf" "Inter-4.1/extras/ttf/Inter-Bold.ttf" 2>/dev/null || true
# Clean up
rm -f inter.zip
rm -rf Inter-4.1/
```

**Step 3: Download JetBrains Mono**

```bash
cd fonts/
curl -L -o jbmono.zip "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip"
unzip -j jbmono.zip "fonts/ttf/JetBrainsMono-Regular.ttf" -d . 2>/dev/null || true
rm -f jbmono.zip
```

**Step 4: Verify font files exist**

```bash
ls -la fonts/*.ttf
```

Expected: Three `.ttf` files present (~100-300KB each).

**Step 5: Commit**

```bash
git add fonts/
git commit -m "chore: add Inter and JetBrains Mono font files"
```

---

### Task 2: Load custom fonts in theme.rs

**Files:**
- Modify: `src/theme.rs` (entire file rewrite)

**Step 1: Add font loading to theme**

Replace the `apply` method in `src/theme.rs` to load custom fonts on first call and set improved text styles. Key points:

- Use `ctx.add_font()` with `FontInsert` + `InsertFontFamily` + `FontPriority::Highest`
- Load Inter-Regular for `FontFamily::Proportional`
- Load JetBrains Mono for `FontFamily::Monospace`
- Use a `static AtomicBool` or check `ctx.fonts()` to avoid re-adding fonts every frame
- Body: 17px, Heading: 30px, Monospace: 15px, Button: 15px, Small: 13px
- Set `style.spacing.item_spacing.y = 6.0` for global spacing improvement

The implementation should look like this in `src/theme.rs`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static FONTS_LOADED: AtomicBool = AtomicBool::new(false);

// In the apply() method, before setting visuals:
if !FONTS_LOADED.swap(true, Ordering::Relaxed) {
    use egui::epaint::text::{FontInsert, InsertFontFamily, FontPriority};
    use egui::FontData;

    ctx.add_font(FontInsert::new(
        "inter",
        FontData::from_static(include_bytes!("../fonts/Inter-Regular.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: FontPriority::Highest,
        }],
    ));

    ctx.add_font(FontInsert::new(
        "jetbrains_mono",
        FontData::from_static(include_bytes!("../fonts/JetBrainsMono-Regular.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Monospace,
            priority: FontPriority::Highest,
        }],
    ));
}
```

Font sizes in the style block:
- `TextStyle::Heading` → 30.0px Proportional
- `TextStyle::Body` → 17.0px Proportional
- `TextStyle::Monospace` → 15.0px Monospace
- `TextStyle::Button` → 15.0px Proportional
- `TextStyle::Small` → 13.0px Proportional

Global spacing:
- `style.spacing.item_spacing = egui::vec2(8.0, 6.0);`

**Step 2: Build and verify**

```bash
cargo build 2>&1
```

Expected: Compiles with no errors. Warnings about unused code are OK.

**Step 3: Run and visually verify**

```bash
cargo run -- test_search.md
```

Expected: Window opens with Inter font for body text and JetBrains Mono in code blocks. Text should look noticeably more professional.

**Step 4: Commit**

```bash
git add src/theme.rs
git commit -m "feat: load Inter and JetBrains Mono custom fonts"
```

---

### Task 3: Fix line height and spacing in render.rs

**Files:**
- Modify: `src/render.rs:172-218` (FormatState::to_text_format method)
- Modify: `src/render.rs:24-39` (heading spacing in render_highlighted_markdown)

**Step 1: Update to_text_format to set line_height**

In `FormatState::to_text_format()`, change the body text size from 14.0 to 17.0, and add a `line_height` field to the returned `TextFormat`:

```rust
fn to_text_format(&self, ui: &egui::Ui) -> TextFormat {
    let size = match self.heading {
        Some(HeadingLevel::H1) => 30.0,
        Some(HeadingLevel::H2) => 26.0,
        Some(HeadingLevel::H3) => 22.0,
        Some(HeadingLevel::H4) => 19.0,
        Some(HeadingLevel::H5) => 17.0,
        Some(HeadingLevel::H6) => 15.0,
        None => 17.0,  // Was 14.0 — match theme body size
    };

    // ... (existing family, color logic stays the same) ...

    let mut format = TextFormat {
        font_id: FontId::new(size, family),
        color,
        line_height: Some(size * 1.35),  // 1.35x line height
        ..Default::default()
    };

    // ... (rest stays the same)
}
```

**Step 2: Update heading spacing**

In `render_highlighted_markdown()`, update the heading event handlers:

```rust
Event::Start(Tag::Heading { level, .. }) => {
    flush_job(ui, &mut job);
    ui.add_space(12.0);  // Was 8.0
    fmt.heading = Some(level);
}
Event::End(TagEnd::Heading(_)) => {
    flush_job(ui, &mut job);
    fmt.heading = None;
    ui.add_space(6.0);  // Was 4.0
}
```

**Step 3: Build and verify**

```bash
cargo build 2>&1
```

Expected: Compiles cleanly.

**Step 4: Run and visually verify with search active**

```bash
cargo run -- test_search.md
```

Open the app, press Ctrl+F to enter search mode and type a query. The search-highlighted text should now show proper 1.35x line spacing and larger body text that matches the non-search view.

**Step 5: Commit**

```bash
git add src/render.rs
git commit -m "feat: increase line height to 1.35x and fix body text size in render"
```

---

### Task 4: Refine toolbar in app.rs

**Files:**
- Modify: `src/app.rs:81-87` (hover zone)
- Modify: `src/app.rs:234-447` (toolbar rendering)

This is the largest task. The entire toolbar rendering block (inside the `if opacity > 0.01` condition) gets rewritten.

**Step 1: Reduce hover detection zone**

In `update_toolbar_visibility()`, change line 83:

```rust
// Before:
self.mouse_in_toolbar_zone = mouse_pos.y < 80.0;
// After:
self.mouse_in_toolbar_zone = mouse_pos.y < 60.0;
```

**Step 2: Rewrite toolbar frame**

Replace the toolbar `Frame` setup to use shadow instead of stroke, and increase corner radius:

```rust
let frame = egui::Frame::NONE
    .fill(bg_color)
    .shadow(egui::Shadow {
        offset: egui::vec2(0.0, 2.0),
        blur: 8.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(40),
    })
    .corner_radius(10.0)
    .inner_margin(egui::Margin {
        left: 10,
        right: 10,
        top: 8,
        bottom: 8,
    });
```

**Step 3: Rewrite toolbar buttons (non-search mode)**

Remove all `ui.group()` wrappers. Use flat buttons with separators:

```rust
ui.horizontal_centered(|ui| {
    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);

    // Navigation
    let back_enabled = self.history_pos > 0;
    let forward_enabled = self.history_pos < self.history.len().saturating_sub(1);

    if ui.add_enabled(back_enabled, egui::Button::new("Back")).clicked() {
        // ... back logic
    }
    if ui.add_enabled(forward_enabled, egui::Button::new("Fwd")).clicked() {
        // ... forward logic
    }

    ui.separator();

    // File operations
    if ui.button("Open").clicked() {
        // ... open file logic
    }

    if !is_narrow {
        ui.separator();
        if ui.button(self.theme.name()).clicked() {
            self.theme.toggle();
        }
    }

    // Push search to the right
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("Search").clicked() {
            self.show_search = true;
        }
    });
});
```

**Step 4: Rewrite search bar mode**

Same flat style without `ui.group()`:

```rust
ui.horizontal_centered(|ui| {
    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 6.0);

    ui.label("Find:");
    let response = ui.add(
        egui::TextEdit::singleline(&mut self.search.query)
            .desired_width(200.0)
            .margin(egui::vec2(8.0, 6.0)),
    );
    if response.changed() {
        if let Some(content) = &self.content {
            self.search.search(content);
        }
        self.scroll_to_current_match();
    }

    ui.label(self.search.match_count());

    ui.separator();

    ui.add_enabled_ui(self.search.has_matches(), |ui| {
        if ui.button("Prev").clicked() {
            self.search.prev_match();
            self.scroll_to_current_match();
        }
        if ui.button("Next").clicked() {
            self.search.next_match();
            self.scroll_to_current_match();
        }
    });

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("Close").clicked() {
            self.show_search = false;
        }
    });
});
```

**Step 5: Build and verify**

```bash
cargo build 2>&1
```

Expected: Compiles cleanly.

**Step 6: Run and visually verify toolbar**

```bash
cargo run -- test_search.md
```

Verify:
- Toolbar appears when hovering near top (within 60px)
- No grouped borders, just clean flat buttons with separators
- No emojis on buttons
- Shadow beneath toolbar
- Search mode (Ctrl+F) looks clean with flat buttons
- Theme toggle still works
- Back/Forward navigation still works

**Step 7: Commit**

```bash
git add src/app.rs
git commit -m "feat: refine toolbar with shadow, flat buttons, no emojis"
```

---

### Task 5: Final verification and cleanup

**Step 1: Full build check**

```bash
cargo build 2>&1
```

Expected: No errors. Acceptable warnings only.

**Step 2: Run with a real markdown file**

```bash
cargo run -- test_search.md
```

Verify all three improvements together:
- Inter font renders for body text
- JetBrains Mono renders in code blocks
- Line spacing is comfortable (1.35x) in search mode
- Toolbar looks polished with shadow and flat buttons
- Theme toggle (Dark/Light) still works
- Search (Ctrl+F) works with new toolbar style
- File open dialog works
- Back/Forward navigation works

**Step 3: Commit any final fixes if needed**
