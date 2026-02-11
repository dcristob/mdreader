use crate::file;
use crate::file::FileWatcher;
use crate::markdown::MarkdownContent;
use crate::search::SearchState;
use crate::theme::Theme;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;

pub struct MdReaderApp {
    pub current_file: Option<PathBuf>,
    pub content: Option<String>,
    pub error: Option<String>,
    pub markdown: Option<MarkdownContent>,
    pub cache: CommonMarkCache,
    pub theme: Theme,
    pub search: SearchState,
    pub show_search: bool,
    pub history: Vec<PathBuf>,
    pub history_pos: usize,
    pub file_watcher: Option<FileWatcher>,
    pub toolbar_visible: bool,
    pub toolbar_opacity: f32,
    pub mouse_in_toolbar_zone: bool,
}

impl Default for MdReaderApp {
    fn default() -> Self {
        Self {
            current_file: None,
            content: None,
            error: None,
            markdown: None,
            cache: CommonMarkCache::default(),
            theme: Theme::default(),
            search: SearchState::default(),
            show_search: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            toolbar_visible: false,
            toolbar_opacity: 0.0,
            mouse_in_toolbar_zone: false,
        }
    }
}

impl MdReaderApp {
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut app = Self {
            current_file: file.clone(),
            content: None,
            error: None,
            markdown: None,
            cache: CommonMarkCache::default(),
            theme: Theme::default(),
            search: SearchState::default(),
            show_search: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            toolbar_visible: false,
            toolbar_opacity: 0.0,
            mouse_in_toolbar_zone: false,
        };

        if let Some(path) = file {
            app.load_file(path);
        }

        app
    }

    fn update_toolbar_visibility(&mut self, ctx: &egui::Context) {
        // Check if mouse is in the top 60px of the window
        if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            self.mouse_in_toolbar_zone = mouse_pos.y < 60.0;
        } else {
            self.mouse_in_toolbar_zone = false;
        }

        // Also show toolbar if search is active
        let should_show = self.mouse_in_toolbar_zone || self.show_search;

        // Smooth animation (1-2 seconds transition)
        let target_opacity = if should_show { 1.0 } else { 0.0 };
        let animation_speed = 0.015; // Adjust for 1-2 second transition

        if self.toolbar_opacity < target_opacity {
            self.toolbar_opacity = (self.toolbar_opacity + animation_speed).min(1.0);
            ctx.request_repaint();
        } else if self.toolbar_opacity > target_opacity {
            self.toolbar_opacity = (self.toolbar_opacity - animation_speed).max(0.0);
            ctx.request_repaint();
        }

        self.toolbar_visible = self.toolbar_opacity > 0.01;
    }

    fn load_file(&mut self, path: PathBuf) {
        if self.history_pos < self.history.len().saturating_sub(1) {
            self.history.truncate(self.history_pos + 1);
        }

        self.current_file = Some(path.clone());
        self.history.push(path.clone());
        self.history_pos = self.history.len() - 1;

        match file::load_file(&path) {
            Ok(content) => {
                self.markdown = Some(crate::markdown::parse(&content));
                self.content = Some(content);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
                self.markdown = None;
            }
        }

        match FileWatcher::new(&path) {
            Ok(watcher) => self.file_watcher = Some(watcher),
            Err(_) => self.file_watcher = None,
        }
    }

    fn load_file_without_history(&mut self, path: PathBuf) {
        self.current_file = Some(path.clone());
        match file::load_file(&path) {
            Ok(content) => {
                self.markdown = Some(crate::markdown::parse(&content));
                self.content = Some(content);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
                self.markdown = None;
            }
        }
    }

    fn handle_link(&mut self, url: &str) {
        use crate::links::{classify_link, LinkType};

        match classify_link(url, self.current_file.as_deref()) {
            LinkType::External(url) => {
                if let Err(e) = crate::links::open_link(&url) {
                    self.error = Some(format!("Failed to open link: {}", e));
                }
            }
            LinkType::Internal(path) => {
                if path.exists() {
                    self.load_file(path);
                } else {
                    self.error = Some(format!("File not found: {}", path.display()));
                }
            }
        }
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update toolbar auto-hide animation
        self.update_toolbar_visibility(ctx);

        if let Some(ref watcher) = self.file_watcher {
            match watcher.receiver.try_recv() {
                Ok(Ok(_event)) => {
                    if let Some(ref path) = self.current_file {
                        if let Ok(content) = file::load_file(path) {
                            self.markdown = Some(crate::markdown::parse(&content));
                            self.content = Some(content);
                        }
                    }
                }
                Ok(Err(_)) | Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.file_watcher = None;
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F)) && ctx.input(|i| i.modifiers.ctrl) {
            self.show_search = !self.show_search;
        }

        self.theme.apply(ctx);

        // Auto-hiding toolbar - fades in/out smoothly (overlays content without scrolling)
        if self.toolbar_opacity > 0.01 {
            let opacity = self.toolbar_opacity;

            // Create semi-transparent background color
            let bg_color = if self.theme == Theme::Dark {
                egui::Color32::from_rgba_premultiplied(30, 30, 30, (opacity * 220.0) as u8)
            } else {
                egui::Color32::from_rgba_premultiplied(240, 240, 240, (opacity * 220.0) as u8)
            };

            let screen_width = ctx.screen_rect().width();

            // Use Area to overlay on top without affecting layout (no scrolling)
            egui::Area::new("toolbar_area".into())
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0))
                .movable(false)
                .show(ctx, |ui| {
                    let frame = egui::Frame::group(ui.style())
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(
                            1.0,
                            ui.visuals()
                                .widgets
                                .noninteractive
                                .bg_stroke
                                .color
                                .linear_multiply(opacity),
                        ))
                        .rounding(8.0);

                    frame.show(ui, |ui| {
                        ui.set_min_width(screen_width - 100.0);
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);

                            // Navigation group
                            ui.group(|ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 8.0);

                                    let back_enabled = self.history_pos > 0;
                                    let forward_enabled =
                                        self.history_pos < self.history.len().saturating_sub(1);

                                    if ui
                                        .add_enabled(back_enabled, egui::Button::new("◀ Back"))
                                        .clicked()
                                    {
                                        if self.history_pos > 0 {
                                            self.history_pos -= 1;
                                            if let Some(path) =
                                                self.history.get(self.history_pos).cloned()
                                            {
                                                self.load_file_without_history(path);
                                            }
                                        }
                                    }

                                    if ui
                                        .add_enabled(
                                            forward_enabled,
                                            egui::Button::new("▶ Forward"),
                                        )
                                        .clicked()
                                    {
                                        if self.history_pos < self.history.len().saturating_sub(1) {
                                            self.history_pos += 1;
                                            if let Some(path) =
                                                self.history.get(self.history_pos).cloned()
                                            {
                                                self.load_file_without_history(path);
                                            }
                                        }
                                    }
                                });
                            });

                            // Flexible space to push groups apart
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add_space(ui.available_width() * 0.2);
                                },
                            );

                            // File operations group
                            ui.group(|ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 8.0);

                                    if ui.button("📂 Open File").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Markdown", &["md", "markdown"])
                                            .add_filter("All Files", &["*"])
                                            .pick_file()
                                        {
                                            self.load_file(path);
                                        }
                                    }
                                });
                            });

                            // Flexible space
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add_space(ui.available_width() * 0.3);
                                },
                            );

                            // Theme toggle group
                            ui.group(|ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 8.0);

                                    if ui.button(format!("🌓 {}", self.theme.name())).clicked() {
                                        self.theme.toggle();
                                    }
                                });
                            });

                            // Flexible space
                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add_space(ui.available_width() * 0.5);
                                },
                            );

                            // Search group
                            ui.group(|ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 8.0);

                                    let search_text = if self.show_search {
                                        "✕ Close Search"
                                    } else {
                                        "🔍 Search (Ctrl+F)"
                                    };

                                    if ui.button(search_text).clicked() {
                                        self.show_search = !self.show_search;
                                    }
                                });
                            });

                            ui.add_space(20.0);
                        });
                    });
                });
        }

        // Links bar
        let links_to_show: Vec<(String, String)> = self
            .markdown
            .as_ref()
            .map(|m| {
                m.links
                    .iter()
                    .map(|l| (l.text.clone(), l.url.clone()))
                    .collect()
            })
            .unwrap_or_default();

        if !links_to_show.is_empty() {
            egui::TopBottomPanel::top("links_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label("Links:");
                    for (text, url) in &links_to_show {
                        if ui.button(text).clicked() {
                            self.handle_link(url);
                        }
                    }
                });
            });
        }

        // Search bar
        if self.show_search {
            egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label("🔍 Find:");

                    let response = ui.text_edit_singleline(&mut self.search.query);
                    if response.changed() {
                        if let Some(content) = &self.content {
                            self.search.search(content);
                        }
                    }

                    ui.label(self.search.match_count());

                    ui.add_enabled_ui(self.search.has_matches(), |ui| {
                        if ui.button("◀ Prev").clicked() {
                            self.search.prev_match();
                        }
                        if ui.button("▶ Next").clicked() {
                            self.search.next_match();
                        }
                    });

                    if ui.button("✕").clicked() {
                        self.show_search = false;
                    }
                });
            });
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            } else if let Some(content) = &self.content {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    CommonMarkViewer::new().show(ui, &mut self.cache, content);
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Welcome to Markdown Viewer");
                    ui.add_space(20.0);
                    ui.label("Open a markdown file to begin");
                    ui.add_space(10.0);
                    ui.label("Use 📂 Open File or drag and drop a file");
                });
            }
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if let Some(ref path) = self.current_file {
                    ui.label(format!("📄 {}", path.display()));
                } else {
                    ui.label("No file open");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if let Some(ref content) = self.content {
                        let lines = content.lines().count();
                        let chars = content.chars().count();
                        ui.label(format!("{} lines, {} chars", lines, chars));
                    }
                });
            });
        });
    }
}
