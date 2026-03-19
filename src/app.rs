use crate::file;
use crate::file::FileWatcher;
use crate::search::SearchState;
use crate::theme::Theme;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;

pub struct MdReaderApp {
    pub current_file: Option<PathBuf>,
    pub content: Option<String>,
    pub error: Option<String>,
    pub cache: CommonMarkCache,
    pub theme: Theme,
    pub search: SearchState,
    pub show_search: bool,
    pub search_focus_requested: bool,
    pub history: Vec<PathBuf>,
    pub history_pos: usize,
    pub file_watcher: Option<FileWatcher>,
    pub toolbar_visible: bool,
    pub toolbar_opacity: f32,
    pub mouse_in_toolbar_zone: bool,
    pub scroll_to_match: bool,
}

impl Default for MdReaderApp {
    fn default() -> Self {
        Self {
            current_file: None,
            content: None,
            error: None,
            cache: CommonMarkCache::default(),
            theme: Theme::default(),
            search: SearchState::default(),
            show_search: false,
            search_focus_requested: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            toolbar_visible: false,
            toolbar_opacity: 0.0,
            mouse_in_toolbar_zone: false,
            scroll_to_match: false,
        }
    }
}

fn is_mdx_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("mdx"))
        .unwrap_or(false)
}

impl MdReaderApp {
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut app = Self {
            current_file: file.clone(),
            content: None,
            error: None,
            cache: CommonMarkCache::default(),
            theme: crate::config::load_theme(),
            search: SearchState::default(),
            show_search: false,
            search_focus_requested: false,
            history: Vec::new(),
            history_pos: 0,
            file_watcher: None,
            toolbar_visible: false,
            toolbar_opacity: 0.0,
            mouse_in_toolbar_zone: false,
            scroll_to_match: false,
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

        // Smooth, natural animation
        let target_opacity = if should_show { 1.0 } else { 0.0 };
        let animation_speed = 0.08; // Faster, more responsive (was 0.015)

        if self.toolbar_opacity < target_opacity {
            self.toolbar_opacity = (self.toolbar_opacity + animation_speed).min(1.0);
            if self.toolbar_opacity < 1.0 {
                ctx.request_repaint();
            }
        } else if self.toolbar_opacity > target_opacity {
            self.toolbar_opacity = (self.toolbar_opacity - animation_speed).max(0.0);
            if self.toolbar_opacity > 0.0 {
                ctx.request_repaint();
            }
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

        crate::config::add_recent_file(&path);

        match file::load_file(&path) {
            Ok(content) => {
                self.content = Some(if is_mdx_file(&path) {
                    file::strip_mdx_imports(&content)
                } else {
                    content
                });
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
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
                self.content = Some(if is_mdx_file(&path) {
                    file::strip_mdx_imports(&content)
                } else {
                    content
                });
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
            }
        }

        match FileWatcher::new(&path) {
            Ok(watcher) => self.file_watcher = Some(watcher),
            Err(_) => self.file_watcher = None,
        }
    }

    fn scroll_to_current_match(&mut self) {
        if self.search.current_match.is_some() {
            self.scroll_to_match = true;
        }
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag-and-drop
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    return Some(path.clone());
                }
            }
            None
        }).map(|path| self.load_file(path));

        // Update toolbar auto-hide animation
        self.update_toolbar_visibility(ctx);

        if let Some(ref watcher) = self.file_watcher {
            match watcher.receiver.try_recv() {
                Ok(Ok(_event)) => {
                    if let Some(ref path) = self.current_file {
                        if let Ok(content) = file::load_file(path) {
                            self.content = Some(if is_mdx_file(path) {
                                file::strip_mdx_imports(&content)
                            } else {
                                content
                            });
                        }
                    }
                }
                Ok(Err(_)) | Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.file_watcher = None;
                }
            }
        }

        // Request continuous repaints during scrolling for smooth animation
        if ctx.input(|i| i.smooth_scroll_delta != egui::Vec2::ZERO) {
            ctx.request_repaint();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.ctrl) {
            self.show_search = !self.show_search;
            if self.show_search {
                self.search_focus_requested = true;
            }
        }

        if self.show_search {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.show_search = false;
            } else if ctx.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.shift) {
                self.search.prev_match();
                self.scroll_to_current_match();
            } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search.next_match();
                self.scroll_to_current_match();
            }
        }

        self.theme.apply(ctx);

        let screen_rect = ctx.input(|i| i.content_rect());
        let screen_width = screen_rect.width();
        let is_narrow = screen_width < 800.0;

        // Overlay toolbar - appears on top of content without reserving space
        let opacity = if self.show_search {
            1.0
        } else {
            self.toolbar_opacity
        };

        if opacity > 0.01 {
            let base_color = if self.theme == Theme::Dark {
                egui::Color32::from_rgb(30, 30, 30)
            } else {
                egui::Color32::from_rgb(245, 245, 245)
            };
            let bg_color = egui::Color32::from_rgba_premultiplied(
                base_color.r(),
                base_color.g(),
                base_color.b(),
                (opacity * 235.0) as u8,
            );

            egui::Area::new("toolbar_area".into())
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 8.0))
                .movable(false)
                .show(ctx, |ui| {
                    let frame = egui::Frame::NONE
                        .fill(bg_color)
                        .shadow(egui::Shadow {
                            offset: [0, 2],
                            blur: 8,
                            spread: 0,
                            color: egui::Color32::from_black_alpha(40),
                        })
                        .corner_radius(10.0)
                        .inner_margin(egui::Margin {
                            left: 10,
                            right: 10,
                            top: 8,
                            bottom: 8,
                        });

                    frame.show(ui, |ui| {
                        let max_width = (screen_width - 40.0).max(400.0);
                        ui.set_max_width(max_width);

                        if self.show_search {
                            // Search Bar Mode
                            ui.horizontal_centered(|ui| {
                                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 3.0);

                                ui.label("Find:");

                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.search.query)
                                        .desired_width(200.0)
                                        .margin(egui::vec2(8.0, 6.0)),
                                );

                                if self.search_focus_requested {
                                    response.request_focus();
                                    self.search_focus_requested = false;
                                }

                                if response.changed() {
                                    if let Some(content) = &self.content {
                                        self.search.search(content);
                                    }
                                    self.scroll_to_current_match();
                                }

                                ui.label(self.search.match_count());

                                ui.separator();

                                ui.add_enabled_ui(self.search.has_matches(), |ui| {
                                    if ui.button("▲ Prev").clicked() {
                                        self.search.prev_match();
                                        self.scroll_to_current_match();
                                    }
                                    if ui.button("▼ Next").clicked() {
                                        self.search.next_match();
                                        self.scroll_to_current_match();
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("✕ Close").clicked() {
                                            self.show_search = false;
                                        }
                                    },
                                );
                            });
                        } else {
                            // Toolbar Mode
                            ui.horizontal_centered(|ui| {
                                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 3.0);

                                // Navigation
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
                                    .add_enabled(forward_enabled, egui::Button::new("Fwd ▶"))
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

                                ui.separator();

                                // File operations
                                if ui.button("📂 Open").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Markdown", &["md", "markdown"])
                                        .add_filter("All Files", &["*"])
                                        .pick_file()
                                    {
                                        self.load_file(path);
                                    }
                                }

                                if !is_narrow {
                                    ui.separator();

                                    // Theme toggle
                                    let theme_label = if self.theme == Theme::Dark {
                                        "🌙 Dark"
                                    } else {
                                        "☀ Light"
                                    };
                                    if ui.button(theme_label).clicked() {
                                        self.theme.toggle();
                                        crate::config::save_theme(self.theme);
                                    }
                                }

                                // Push search to the right
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("🔍 Search").clicked() {
                                            self.show_search = true;
                                            self.search_focus_requested = true;
                                        }
                                    },
                                );
                            });
                        }
                    });
                });
        }

        // Main content area — top margin matches toolbar height so content
        // starts below the overlay; left/right/bottom provide reading comfort.
        let content_margin = egui::Margin {
            left: 24,
            right: 24,
            top: 64,
            bottom: 40,
        };
        let content_frame = egui::Frame::central_panel(ctx.style().as_ref())
            .inner_margin(content_margin);
        let mut pending_nav: Option<std::path::PathBuf> = None;
        egui::CentralPanel::default().frame(content_frame).show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            } else if let Some(content) = &self.content {
                let scroll_area = egui::ScrollArea::vertical()
                    .wheel_scroll_multiplier(egui::vec2(1.0, 2.0));

                let scroll_to = self.scroll_to_match;
                self.scroll_to_match = false;

                scroll_area.show(ui, |ui: &mut egui::Ui| {
                    let base_dir = self.current_file.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf()));
                    if self.show_search && self.search.has_matches() {
                        let action = crate::render::render_highlighted_markdown(
                            ui,
                            content,
                            &self.search,
                            scroll_to,
                            base_dir.as_deref(),
                        );
                        if let Some(action) = action {
                            match action {
                                crate::render::LinkAction::OpenUrl(url) => {
                                    let _ = open::that(&url);
                                }
                                crate::render::LinkAction::NavigateFile(path) => {
                                    pending_nav = Some(path);
                                }
                            }
                        }
                    } else {
                        CommonMarkViewer::new().show(ui, &mut self.cache, content);
                    }
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Welcome to Markdown Viewer");
                    ui.add_space(20.0);
                    ui.label("Open a markdown file to begin");
                    ui.add_space(10.0);
                    ui.label("Use Open, drag and drop, or Ctrl+F to search");

                    let recent = crate::config::load_recent_files();
                    if !recent.is_empty() {
                        ui.add_space(30.0);
                        ui.heading("Recent Files");
                        ui.add_space(10.0);
                        let mut open_path = None;
                        for path in &recent {
                            let display = path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| path.display().to_string());
                            let label = format!("📄 {}", display);
                            if ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&label)
                                        .color(egui::Color32::from_rgb(66, 133, 244))
                                )
                                .selectable(false)
                                .sense(egui::Sense::click())
                            ).on_hover_text(path.display().to_string()).clicked() {
                                open_path = Some(path.clone());
                            }
                        }
                        if let Some(path) = open_path {
                            pending_nav = Some(path);
                        }
                    }
                });
            }
        });

        // Handle link navigation after panel rendering (avoids borrow conflicts)
        if let Some(path) = pending_nav {
            self.load_file(path);
        }

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let small_font = egui::FontId::proportional(11.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if let Some(ref path) = self.current_file {
                    ui.label(egui::RichText::new(format!("📄 {}", path.display())).font(small_font.clone()));
                } else {
                    ui.label(egui::RichText::new("No file open").font(small_font.clone()));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if let Some(ref content) = self.content {
                        let lines = content.lines().count();
                        let chars = content.chars().count();
                        ui.label(egui::RichText::new(format!("{} lines, {} chars", lines, chars)).font(small_font));
                    }
                });
            });
        });
    }
}
