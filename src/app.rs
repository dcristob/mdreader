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
    pub zoom: f32,
    pub search: SearchState,
    pub show_search: bool,
    pub history: Vec<PathBuf>,
    pub history_pos: usize,
    pub file_watcher: Option<FileWatcher>,
    zoom_changed: bool,
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
            zoom: 1.2,
            search: SearchState::default(),
            show_search: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            zoom_changed: true,
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
            zoom: 1.2,
            search: SearchState::default(),
            show_search: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            zoom_changed: true,
        };

        if let Some(path) = file {
            app.load_file(path);
        }

        app
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

    fn apply_zoom(&mut self, ctx: &egui::Context) {
        if self.zoom_changed {
            let mut style = (*ctx.style()).clone();
            style.text_styles = style
                .text_styles
                .into_iter()
                .map(|(id, font)| (id, egui::FontId::new(font.size * self.zoom, font.family)))
                .collect();
            ctx.set_style(style);
            self.zoom_changed = false;
        }
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        self.apply_zoom(ctx);

        // Main toolbar with grouped buttons and professional styling
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // Navigation group
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 6.0);

                        let back_enabled = self.history_pos > 0;
                        let forward_enabled =
                            self.history_pos < self.history.len().saturating_sub(1);

                        if ui
                            .add_enabled(back_enabled, egui::Button::new("◀ Back"))
                            .clicked()
                        {
                            if self.history_pos > 0 {
                                self.history_pos -= 1;
                                if let Some(path) = self.history.get(self.history_pos).cloned() {
                                    self.load_file_without_history(path);
                                }
                            }
                        }

                        if ui
                            .add_enabled(forward_enabled, egui::Button::new("▶ Forward"))
                            .clicked()
                        {
                            if self.history_pos < self.history.len().saturating_sub(1) {
                                self.history_pos += 1;
                                if let Some(path) = self.history.get(self.history_pos).cloned() {
                                    self.load_file_without_history(path);
                                }
                            }
                        }
                    });
                });

                ui.add_space(12.0);

                // File operations group
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 6.0);

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

                ui.add_space(12.0);

                // View controls group
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 6.0);

                        if ui.button(format!("🌓 {}", self.theme.name())).clicked() {
                            self.theme.toggle();
                        }

                        ui.separator();

                        // Zoom controls with proper debouncing
                        let zoom_out_btn = ui.add_enabled(
                            self.zoom > 0.6,
                            egui::Button::new("−").min_size(egui::vec2(28.0, 0.0)),
                        );
                        if zoom_out_btn.clicked() {
                            self.zoom = (self.zoom - 0.1).max(0.5);
                            self.zoom_changed = true;
                        }

                        ui.label(format!("{:.0}%", self.zoom * 100.0))
                            .on_hover_text("Current zoom level");

                        let zoom_in_btn = ui.add_enabled(
                            self.zoom < 2.9,
                            egui::Button::new("+").min_size(egui::vec2(28.0, 0.0)),
                        );
                        if zoom_in_btn.clicked() {
                            self.zoom = (self.zoom + 0.1).min(3.0);
                            self.zoom_changed = true;
                        }
                    });
                });

                ui.add_space(12.0);

                // Search group
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 6.0);

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

                ui.add_space(8.0);
            });
            ui.add_space(4.0);
        });

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
