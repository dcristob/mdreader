use crate::search::SearchState;
use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontFamily, FontId, Stroke};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;
use std::sync::OnceLock;
use syntect::highlighting::{ThemeSet, Style as SyntectStyle};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

/// Returns true if the markdown contains at least one GFM table.
/// Used to route table-containing docs to our custom renderer because
/// egui_commonmark's Grid-based table layout doesn't wrap cell text.
pub fn contains_table(content: &str) -> bool {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    Parser::new_ext(content, options)
        .any(|event| matches!(event, Event::Start(Tag::Table(_))))
}

/// Link click action returned from rendering, to be handled by the caller.
pub enum LinkAction {
    /// Open an external URL in the system browser
    OpenUrl(String),
    /// Navigate to a relative markdown file
    NavigateFile(std::path::PathBuf),
}

/// Renders markdown with search match highlighting.
/// Uses pulldown-cmark for parsing and egui LayoutJob for styled text
/// with inline background-color highlights on search matches.
/// When `scroll_to_match` is true, the label containing the current match
/// will call `scroll_to_me` for pixel-accurate scrolling.
pub fn render_highlighted_markdown(
    ui: &mut egui::Ui,
    content: &str,
    search: &SearchState,
    scroll_to_match: bool,
    base_dir: Option<&std::path::Path>,
) -> Option<LinkAction> {
    let is_dark = ui.visuals().dark_mode;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(content, options).into_offset_iter();

    let mut job = LayoutJob::default();
    let mut fmt = FormatState::default();
    let mut in_code_block = false;
    let mut code_block_text = String::new();
    let mut code_block_source_start: usize = 0;
    let mut code_block_lang = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut need_list_marker = false;
    let mut job_has_current_match = false;
    let mut current_link_url: Option<String> = None;
    let mut link_action: Option<LinkAction> = None;
    let mut table_state: Option<TableState> = None;
    let mut table_counter: usize = 0;

    for (event, range) in parser {
        // Route every event inside a table to the table accumulator so
        // cell text doesn't bleed into the main flow and so the table
        // structure is preserved for the deferred grid render.
        if let Some(ts) = table_state.as_mut() {
            match event {
                Event::Start(Tag::TableHead) => ts.in_header = true,
                Event::End(TagEnd::TableHead) => ts.in_header = false,
                Event::Start(Tag::TableRow) | Event::Start(Tag::TableCell) => {}
                Event::End(TagEnd::TableRow) => {
                    let row = std::mem::take(&mut ts.current_row);
                    ts.rows.push(row);
                }
                Event::End(TagEnd::TableCell) => {
                    let cell = std::mem::take(&mut ts.current_cell);
                    if ts.in_header {
                        ts.header.push(cell);
                    } else {
                        ts.current_row.push(cell);
                    }
                }
                Event::End(TagEnd::Table) => {
                    let finished = table_state.take().unwrap();
                    table_counter += 1;
                    render_table(ui, finished, table_counter, scroll_to_match);
                    ui.add_space(8.0);
                }
                Event::Start(Tag::Strong) => fmt.bold = true,
                Event::End(TagEnd::Strong) => fmt.bold = false,
                Event::Start(Tag::Emphasis) => fmt.italic = true,
                Event::End(TagEnd::Emphasis) => fmt.italic = false,
                Event::Start(Tag::Strikethrough) => fmt.strikethrough = true,
                Event::End(TagEnd::Strikethrough) => fmt.strikethrough = false,
                Event::Text(text) => {
                    let mut cell_fmt = fmt.clone();
                    if ts.in_header {
                        cell_fmt.bold = true;
                    }
                    if append_highlighted_text(
                        &mut ts.current_cell,
                        &text,
                        range,
                        content,
                        search,
                        &cell_fmt,
                        ui,
                    ) {
                        ts.has_current_match = true;
                    }
                }
                Event::Code(code) => {
                    let mut code_fmt = fmt.clone();
                    code_fmt.code = true;
                    if ts.in_header {
                        code_fmt.bold = true;
                    }
                    if append_highlighted_text(
                        &mut ts.current_cell,
                        &code,
                        range,
                        content,
                        search,
                        &code_fmt,
                        ui,
                    ) {
                        ts.has_current_match = true;
                    }
                }
                Event::SoftBreak => {
                    let tf = fmt.to_text_format(ui);
                    ts.current_cell.append(" ", 0.0, tf);
                }
                Event::HardBreak => {
                    let tf = fmt.to_text_format(ui);
                    ts.current_cell.append("\n", 0.0, tf);
                }
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::Table(_)) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                table_state = Some(TableState::default());
            }
            // Block-level elements
            Event::Start(Tag::Heading { level, .. }) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                ui.add_space(12.0);
                fmt.heading = Some(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                fmt.heading = None;
                ui.add_space(6.0);
            }
            Event::Start(Tag::Paragraph) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
            }
            Event::End(TagEnd::Paragraph) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                ui.add_space(8.0);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                in_code_block = true;
                code_block_text.clear();
                code_block_source_start = range.start;
                code_block_lang = match &kind {
                    CodeBlockKind::Fenced(lang) => lang.split_whitespace().next().unwrap_or("").to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let code_text = std::mem::take(&mut code_block_text);
                render_code_block(ui, &code_text, &code_block_lang, search, content, code_block_source_start, scroll_to_match, is_dark);
                ui.add_space(8.0);
            }
            Event::Start(Tag::List(start)) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                list_stack.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
                if list_stack.is_empty() {
                    ui.add_space(8.0);
                }
            }
            Event::Start(Tag::Item) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                need_list_marker = true;
            }
            Event::End(TagEnd::Item) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
            }
            Event::Rule => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                ui.separator();
            }

            // Inline formatting
            Event::Start(Tag::Strong) => fmt.bold = true,
            Event::End(TagEnd::Strong) => fmt.bold = false,
            Event::Start(Tag::Emphasis) => fmt.italic = true,
            Event::End(TagEnd::Emphasis) => fmt.italic = false,
            Event::Start(Tag::Strikethrough) => fmt.strikethrough = true,
            Event::End(TagEnd::Strikethrough) => fmt.strikethrough = false,
            Event::Start(Tag::Link { dest_url, .. }) => {
                fmt.link = true;
                current_link_url = Some(dest_url.to_string());
                // Flush before link so we can make the link text clickable
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
            }
            Event::End(TagEnd::Link) => {
                // Flush the link text and make it clickable
                if !job.text.is_empty() {
                    let mut j = std::mem::take(&mut job);
                    j.wrap.max_width = ui.available_width();
                    let response = ui.add(
                        egui::Label::new(j)
                            .selectable(true)
                            .sense(egui::Sense::click()),
                    );
                    if scroll_to_match && job_has_current_match {
                        response.scroll_to_me(Some(egui::Align::Center));
                    }
                    if response.clicked() {
                        if let Some(ref url) = current_link_url {
                            link_action = Some(resolve_link(url, base_dir));
                        }
                    }
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    job_has_current_match = false;
                }
                fmt.link = false;
                current_link_url = None;
            }

            // Text content
            Event::Text(text) => {
                if in_code_block {
                    code_block_text.push_str(&text);
                    continue;
                }

                if need_list_marker {
                    let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                    let marker = if let Some(Some(n)) = list_stack.last_mut() {
                        let m = format!("{}{}. ", indent, n);
                        *n += 1;
                        m
                    } else {
                        format!("{}• ", indent)
                    };
                    let marker_tf = fmt.to_text_format(ui);
                    job.append(&marker, 0.0, marker_tf);
                    need_list_marker = false;
                }

                if append_highlighted_text(&mut job, &text, range, content, search, &fmt, ui) {
                    job_has_current_match = true;
                }
            }
            Event::Code(code) => {
                let mut code_fmt = fmt.clone();
                code_fmt.code = true;
                if append_highlighted_text(&mut job, &code, range, content, search, &code_fmt, ui)
                {
                    job_has_current_match = true;
                }
            }
            Event::SoftBreak => {
                let tf = fmt.to_text_format(ui);
                job.append(" ", 0.0, tf);
            }
            Event::HardBreak => {
                job.append("\n", 0.0, fmt.to_text_format(ui));
            }

            _ => {}
        }
    }

    flush_job(ui, &mut job, scroll_to_match && job_has_current_match);

    link_action
}

fn resolve_link(url: &str, base_dir: Option<&std::path::Path>) -> LinkAction {
    // External URLs: open in browser
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("mailto:") {
        return LinkAction::OpenUrl(url.to_string());
    }

    // Relative paths: resolve against the current file's directory
    if let Some(base) = base_dir {
        let clean_url = url.split('#').next().unwrap_or(url);
        let target = base.join(clean_url);
        if clean_url.ends_with(".md") || clean_url.ends_with(".markdown") || clean_url.ends_with(".mdx") {
            return LinkAction::NavigateFile(target);
        }
    }

    LinkAction::OpenUrl(url.to_string())
}

fn render_code_block(
    ui: &mut egui::Ui,
    code_text: &str,
    lang: &str,
    search: &SearchState,
    content: &str,
    source_start: usize,
    scroll_to_match: bool,
    is_dark: bool,
) {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let mut code_job = LayoutJob::default();
            let mut has_current = false;

            let ss = syntax_set();
            let ts = theme_set();
            let theme_name = if is_dark { "base16-ocean.dark" } else { "base16-ocean.light" };
            let theme = &ts.themes[theme_name];

            let syntax = if !lang.is_empty() {
                ss.find_syntax_by_token(lang)
            } else {
                None
            }
            .unwrap_or_else(|| ss.find_syntax_plain_text());

            // Compute search match ranges relative to this code block
            let (src_pos, search_matches) = if search.has_matches() {
                let source_slice = &content[source_start..];
                let text_offset = source_slice.find(code_text).unwrap_or(0);
                let sp = source_start + text_offset;
                // Pre-compute which matches overlap this block
                let matches: Vec<(usize, usize, bool)> = search
                    .matches
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.end > sp && m.start < sp + code_text.len())
                    .map(|(idx, m)| {
                        let start = m.start.saturating_sub(sp).min(code_text.len());
                        let end = (m.end - sp).min(code_text.len());
                        let is_current = search.current_match == Some(idx);
                        (start, end, is_current)
                    })
                    .collect();
                (sp, matches)
            } else {
                (0, Vec::new())
            };

            let font_size = 15.0;
            let font_id = FontId::new(font_size, FontFamily::Monospace);

            let mut highlighter = HighlightLines::new(syntax, theme);
            let mut char_offset: usize = 0; // byte offset within code_text

            for line in syntect::util::LinesWithEndings::from(code_text) {
                let regions = highlighter
                    .highlight_line(line, ss)
                    .unwrap_or_default();

                for (style, text) in regions {
                    let seg_start = char_offset;
                    let seg_end = char_offset + text.len();

                    if search_matches.is_empty() {
                        // No search — just syntax colors
                        let tf = syntect_style_to_format(&style, &font_id, ui);
                        code_job.append(text, 0.0, tf);
                    } else {
                        // Interleave syntax colors with search highlights
                        let mut pos = 0usize;
                        for &(m_start, m_end, is_current) in &search_matches {
                            if m_end <= seg_start || m_start >= seg_end {
                                continue;
                            }
                            let overlap_start = if m_start > seg_start { m_start - seg_start } else { 0 };
                            let overlap_end = if m_end < seg_end { m_end - seg_start } else { text.len() };
                            let overlap_start = overlap_start.min(text.len());
                            let overlap_end = overlap_end.min(text.len());

                            if overlap_start > pos {
                                let tf = syntect_style_to_format(&style, &font_id, ui);
                                code_job.append(&text[pos..overlap_start], 0.0, tf);
                            }
                            if overlap_start < overlap_end {
                                if is_current {
                                    has_current = true;
                                }
                                let mut tf = syntect_style_to_format(&style, &font_id, ui);
                                tf.background = if is_current {
                                    Color32::from_rgb(255, 165, 0)
                                } else {
                                    Color32::from_rgb(255, 255, 0)
                                };
                                tf.color = Color32::BLACK;
                                code_job.append(&text[overlap_start..overlap_end], 0.0, tf);
                            }
                            pos = overlap_end;
                        }
                        if pos < text.len() {
                            let tf = syntect_style_to_format(&style, &font_id, ui);
                            code_job.append(&text[pos..], 0.0, tf);
                        }
                    }

                    char_offset = seg_end;
                }
            }

            // Suppress the unused variable warning
            let _ = src_pos;

            code_job.wrap.max_width = ui.available_width();
            let response = ui.label(code_job);
            if scroll_to_match && has_current {
                response.scroll_to_me(Some(egui::Align::Center));
            }
        });
}

fn syntect_style_to_format(style: &SyntectStyle, font_id: &FontId, _ui: &egui::Ui) -> TextFormat {
    let fg = style.foreground;
    TextFormat {
        font_id: font_id.clone(),
        color: Color32::from_rgb(fg.r, fg.g, fg.b),
        line_height: Some(font_id.size * 1.35),
        ..Default::default()
    }
}

#[derive(Clone, Default)]
struct FormatState {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    code: bool,
    heading: Option<HeadingLevel>,
    link: bool,
}

impl FormatState {
    fn to_text_format(&self, ui: &egui::Ui) -> TextFormat {
        let size = match self.heading {
            Some(HeadingLevel::H1) => 30.0,
            Some(HeadingLevel::H2) => 26.0,
            Some(HeadingLevel::H3) => 22.0,
            Some(HeadingLevel::H4) => 19.0,
            Some(HeadingLevel::H5) => 17.0,
            Some(HeadingLevel::H6) => 15.0,
            None => 17.0,
        };

        let family = if self.code {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        };

        let color = if self.link {
            Color32::from_rgb(66, 133, 244)
        } else {
            ui.visuals().text_color()
        };

        let mut format = TextFormat {
            font_id: FontId::new(size, family),
            color,
            line_height: Some(size * 1.35),
            ..Default::default()
        };

        if self.italic {
            format.italics = true;
        }

        if self.strikethrough {
            format.strikethrough = Stroke::new(1.0, color);
        }

        if self.link {
            format.underline = Stroke::new(1.0, color);
        }

        if self.code && self.heading.is_none() {
            format.background = ui.visuals().code_bg_color;
        }

        format
    }

    fn to_text_format_highlighted(&self, ui: &egui::Ui, is_current: bool) -> TextFormat {
        let mut fmt = self.to_text_format(ui);
        fmt.background = if is_current {
            Color32::from_rgb(255, 165, 0) // Orange for current match
        } else {
            Color32::from_rgb(255, 255, 0) // Yellow for other matches
        };
        fmt.color = Color32::BLACK;
        fmt
    }
}

#[derive(Default)]
struct TableState {
    in_header: bool,
    header: Vec<LayoutJob>,
    rows: Vec<Vec<LayoutJob>>,
    current_row: Vec<LayoutJob>,
    current_cell: LayoutJob,
    has_current_match: bool,
}

fn render_table(
    ui: &mut egui::Ui,
    state: TableState,
    table_id: usize,
    scroll_to_match: bool,
) {
    let TableState {
        header,
        rows,
        has_current_match,
        ..
    } = state;

    if header.is_empty() && rows.is_empty() {
        return;
    }

    let n_cols = header
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if n_cols == 0 {
        return;
    }

    // Estimate horizontal overhead from Frame::group padding plus
    // inter-column spacing, so cells share the remaining width evenly.
    let total_width = ui.available_width();
    let item_spacing = ui.style().spacing.item_spacing.x;
    let frame_overhead = 24.0;
    let inter_col = item_spacing * (n_cols.saturating_sub(1) as f32);
    let usable = (total_width - frame_overhead - inter_col).max(120.0);
    let col_width = (usable / n_cols as f32).max(60.0);

    // Build the cells without using egui::Grid: Grid sets the ambient
    // wrap_mode to Extend (horizontal layout), which makes Label override
    // any per-job wrap.max_width with f32::INFINITY. Instead, render each
    // row as a horizontal strip of fixed-width top-down sub-uis, where
    // the inner vertical layout's default wrap_mode is Wrap.
    let _ = table_id; // unused now that Grid is gone
    let stripe_color = ui.visuals().faint_bg_color;

    let render_cell = |ui: &mut egui::Ui, cell: LayoutJob, header: bool| {
        ui.allocate_ui_with_layout(
            egui::vec2(col_width, 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_min_width(col_width);
                ui.set_max_width(col_width);
                let mut label = egui::Label::new(cell).wrap();
                if header {
                    label = label.selectable(true);
                }
                ui.add(label);
            },
        );
    };

    let frame_response = egui::Frame::group(ui.style()).show(ui, |ui| {
        // Header row
        ui.horizontal_top(|ui| {
            let n_header = header.len();
            for cell in header {
                render_cell(ui, cell, true);
            }
            for _ in n_header..n_cols {
                render_cell(ui, LayoutJob::default(), true);
            }
        });
        ui.add_space(2.0);
        ui.separator();

        // Data rows with manual zebra striping
        for (idx, row) in rows.into_iter().enumerate() {
            let row_frame = if idx % 2 == 1 {
                egui::Frame::NONE.fill(stripe_color)
            } else {
                egui::Frame::NONE
            };
            row_frame.show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let n_cells = row.len();
                    for cell in row {
                        render_cell(ui, cell, false);
                    }
                    for _ in n_cells..n_cols {
                        render_cell(ui, LayoutJob::default(), false);
                    }
                });
            });
        }
    });

    if scroll_to_match && has_current_match {
        frame_response
            .response
            .scroll_to_me(Some(egui::Align::Center));
    }
}

fn flush_job(ui: &mut egui::Ui, job: &mut LayoutJob, scroll_to_me: bool) {
    if !job.text.is_empty() {
        let mut j = std::mem::take(job);
        j.wrap.max_width = ui.available_width();
        let response = ui.label(j);
        if scroll_to_me {
            response.scroll_to_me(Some(egui::Align::Center));
        }
    }
}

/// Appends text to a LayoutJob with search match highlighting.
/// Returns `true` if this text segment contains the current (active) match.
fn append_highlighted_text(
    job: &mut LayoutJob,
    text: &str,
    source_range: Range<usize>,
    content: &str,
    search: &SearchState,
    format_state: &FormatState,
    ui: &egui::Ui,
) -> bool {
    if !search.has_matches() || text.is_empty() {
        let tf = format_state.to_text_format(ui);
        job.append(text, 0.0, tf);
        return false;
    }

    let src_start = source_range.start;
    let src_end = source_range.end;

    // Find where the parsed text starts within the source range.
    // Handles inline code (backticks stripped), escapes, etc.
    let source_slice = &content[src_start..src_end.min(content.len())];
    let text_offset = source_slice.find(text).unwrap_or(0);
    let adjusted_start = src_start + text_offset;
    let adjusted_end = adjusted_start + text.len();

    let mut pos = 0usize;
    let mut contains_current = false;

    for (idx, m) in search.matches.iter().enumerate() {
        // Skip matches outside this text segment
        if m.end <= adjusted_start || m.start >= adjusted_end {
            continue;
        }

        let overlap_start = if m.start > adjusted_start {
            m.start - adjusted_start
        } else {
            0
        };
        let overlap_end = if m.end < adjusted_end {
            m.end - adjusted_start
        } else {
            text.len()
        };
        let overlap_start = overlap_start.min(text.len());
        let overlap_end = overlap_end.min(text.len());

        // Non-highlighted text before this match
        if overlap_start > pos {
            let tf = format_state.to_text_format(ui);
            job.append(&text[pos..overlap_start], 0.0, tf);
        }

        // Highlighted match
        if overlap_start < overlap_end {
            let is_current = search.current_match == Some(idx);
            if is_current {
                contains_current = true;
            }
            let hl_fmt = format_state.to_text_format_highlighted(ui, is_current);
            job.append(&text[overlap_start..overlap_end], 0.0, hl_fmt);
        }

        pos = overlap_end;
    }

    // Remaining non-highlighted text
    if pos < text.len() {
        let tf = format_state.to_text_format(ui);
        job.append(&text[pos..], 0.0, tf);
    }

    contains_current
}
