use crate::search::SearchState;
use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontFamily, FontId, Stroke};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::ops::Range;

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
) {
    let parser = Parser::new(content).into_offset_iter();

    let mut job = LayoutJob::default();
    let mut fmt = FormatState::default();
    let mut in_code_block = false;
    let mut code_block_text = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut need_list_marker = false;
    let mut job_has_current_match = false;

    for (event, range) in parser {
        match event {
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
            Event::Start(Tag::CodeBlock(_)) => {
                flush_job(ui, &mut job, scroll_to_match && job_has_current_match);
                job_has_current_match = false;
                in_code_block = true;
                code_block_text.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let code_text = std::mem::take(&mut code_block_text);
                render_code_block(ui, &code_text, search, content, scroll_to_match);
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
            Event::Start(Tag::Link { .. }) => fmt.link = true,
            Event::End(TagEnd::Link) => fmt.link = false,

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
}

fn render_code_block(
    ui: &mut egui::Ui,
    code_text: &str,
    search: &SearchState,
    content: &str,
    scroll_to_match: bool,
) {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().extreme_bg_color)
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let mut code_job = LayoutJob::default();
            let fmt = FormatState {
                code: true,
                ..Default::default()
            };

            let mut has_current = false;
            if !search.has_matches() {
                let tf = fmt.to_text_format(ui);
                code_job.append(code_text, 0.0, tf);
            } else {
                // Find this code block's position in the source to highlight matches
                // We search for the code text in the full content
                if let Some(src_pos) = content.find(code_text) {
                    let fake_range = src_pos..src_pos + code_text.len();
                    has_current = append_highlighted_text(
                        &mut code_job, code_text, fake_range, content, search, &fmt, ui,
                    );
                } else {
                    let tf = fmt.to_text_format(ui);
                    code_job.append(code_text, 0.0, tf);
                }
            }

            code_job.wrap.max_width = ui.available_width();
            let response = ui.label(code_job);
            if scroll_to_match && has_current {
                response.scroll_to_me(Some(egui::Align::Center));
            }
        });
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
