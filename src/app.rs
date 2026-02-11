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
            match file::load_file(&path) {
                Ok(content) => app.content = Some(content),
                Err(e) => app.error = Some(e.to_string()),
            }
        }

        app
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
