use crate::file;
use eframe::egui;
use std::path::PathBuf;

pub struct MdReaderApp {
    pub current_file: Option<PathBuf>,
    pub content: Option<String>,
    pub error: Option<String>,
}

impl Default for MdReaderApp {
    fn default() -> Self {
        Self {
            current_file: None,
            content: None,
            error: None,
        }
    }
}

impl MdReaderApp {
    pub fn new(file: Option<PathBuf>) -> Self {
        let mut app = Self {
            current_file: file.clone(),
            content: None,
            error: None,
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
                self.content = Some(content);
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
            }
        }
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            } else if let Some(content) = &self.content {
                ui.monospace(content);
            } else {
                ui.label("Open a markdown file to begin");
            }
        });
    }
}
