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

        let screen_rect = ctx.screen_rect();
        let screen_width = screen_rect.width();
        let is_narrow = screen_width < 800.0;

        // Main content area - uses full screen (no space reserved for toolbar)
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
                    ui.label("Hover at the top to show toolbar");
                });
            }
        });

        // Overlay toolbar - appears on top of content without reserving space
        let opacity = if self.show_search {
            1.0
        } else {
            self.toolbar_opacity
        };

        if opacity > 0.01 {
            let bg_color = if self.theme == Theme::Dark {
                egui::Color32::from_rgba_premultiplied(35, 35, 35, (opacity * 245.0) as u8)
            } else {
                egui::Color32::from_rgba_premultiplied(250, 250, 250, (opacity * 245.0) as u8)
            };

            egui::Area::new("toolbar_area".into())
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 8.0))
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
                        .rounding(8.0)
                        .inner_margin(egui::Margin::same(12.0));

                    frame.show(ui, |ui| {
                        ui.set_min_width((screen_width * 0.85).min(1200.0));

                        ui.horizontal_centered(|ui| {
                            ui.add_space(16.0);

                            if self.show_search {
                                // Search Bar Mode - all vertically centered
                                ui.horizontal_centered(|ui| {
                                    ui.label("🔍 Find:");

                                    let response = ui.add(
                                        egui::TextEdit::singleline(&mut self.search.query)
                                            .desired_width(200.0)
                                            .margin(egui::vec2(8.0, 6.0)),
                                    );

                                    if response.changed() {
                                        if let Some(content) = &self.content {
                                            self.search.search(content);
                                        }
                                    }

                                    ui.label(self.search.match_count());

                                    // Navigation group
                                    ui.group(|ui| {
                                        ui.horizontal_centered(|ui| {
                                            ui.style_mut().spacing.button_padding =
                                                egui::vec2(10.0, 6.0);

                                            ui.add_enabled_ui(self.search.has_matches(), |ui| {
                                                if ui.button("◀").clicked() {
                                                    self.search.prev_match();
                                                }
                                                if ui.button("▶").clicked() {
                                                    self.search.next_match();
                                                }
                                            });
                                        });
                                    });
                                });

                                // Close button group (same style as search button)
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.group(|ui| {
                                            ui.horizontal_centered(|ui| {
                                                ui.style_mut().spacing.button_padding =
                                                    egui::vec2(10.0, 6.0);

                                                if ui.button("✕ Close").clicked() {
                                                    self.show_search = false;
                                                }
                                            });
                                        });
                                    },
                                );
                            } else {
                                // Toolbar Mode - all vertically centered
                                ui.horizontal_centered(|ui| {
                                    // Navigation group
                                    ui.group(|ui| {
                                        ui.horizontal_centered(|ui| {
                                            ui.style_mut().spacing.button_padding =
                                                egui::vec2(12.0, 8.0);

                                            let back_enabled = self.history_pos > 0;
                                            let forward_enabled = self.history_pos
                                                < self.history.len().saturating_sub(1);

                                            if ui
                                                .add_enabled(
                                                    back_enabled,
                                                    egui::Button::new("◀ Back"),
                                                )
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
                                                if self.history_pos
                                                    < self.history.len().saturating_sub(1)
                                                {
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

                                    ui.add_space(20.0);

                                    // File operations group
                                    ui.group(|ui| {
                                        ui.horizontal_centered(|ui| {
                                            ui.style_mut().spacing.button_padding =
                                                egui::vec2(12.0, 8.0);

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

                                    if !is_narrow {
                                        ui.add_space(20.0);

                                        // Theme toggle group
                                        ui.group(|ui| {
                                            ui.horizontal_centered(|ui| {
                                                ui.style_mut().spacing.button_padding =
                                                    egui::vec2(12.0, 8.0);

                                                if ui
                                                    .button(format!("🌓 {}", self.theme.name()))
                                                    .clicked()
                                                {
                                                    self.theme.toggle();
                                                }
                                            });
                                        });
                                    }
                                });

                                // Push search to the right
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.group(|ui| {
                                            ui.horizontal_centered(|ui| {
                                                ui.style_mut().spacing.button_padding =
                                                    egui::vec2(12.0, 8.0);

                                                if ui.button("🔍 Search").clicked() {
                                                    self.show_search = true;
                                                }
                                            });
                                        });
                                    },
                                );
                            }

                            ui.add_space(16.0);
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
