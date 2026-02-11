use crate::file;
use crate::markdown::MarkdownContent;
use crate::theme::Theme;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::path::PathBuf;

pub struct MdReaderApp {
    pub current_file: Option<PathBuf>,
    pub content: Option<String>,
    pub error: Option<String>,
    pub markdown: Option<MarkdownContent>,
    pub cache: CommonMarkCache,
    pub theme: Theme,
    pub zoom: f32,
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
            zoom: 1.0,
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
            zoom: 1.0,
        };

        if let Some(path) = file {
            app.load_file(path);
        }

        app
    }

    fn load_file(&mut self, path: PathBuf) {
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
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.apply(ctx);

        let mut style = (*ctx.style()).clone();
        style.text_styles = style
            .text_styles
            .into_iter()
            .map(|(id, font)| (id, egui::FontId::new(font.size * self.zoom, font.family)))
            .collect();
        ctx.set_style(style);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Markdown", &["md", "markdown"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        self.load_file(path);
                    }
                }
                ui.separator();
                if ui.button(format!("Theme: {}", self.theme.name())).clicked() {
                    self.theme.toggle();
                }
                ui.separator();
                if ui.button("-").clicked() && self.zoom > 0.5 {
                    self.zoom -= 0.1;
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                if ui.button("+").clicked() && self.zoom < 3.0 {
                    self.zoom += 0.1;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            } else if let Some(content) = &self.content {
                CommonMarkViewer::new().show(ui, &mut self.cache, content);
            } else {
                ui.label("Open a markdown file to begin");
            }
        });
    }
}
