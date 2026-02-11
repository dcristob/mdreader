use eframe::egui;

pub struct MdReaderApp {
    pub current_file: Option<std::path::PathBuf>,
}

impl Default for MdReaderApp {
    fn default() -> Self {
        Self { current_file: None }
    }
}

impl eframe::App for MdReaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Markdown Viewer");
            ui.label("Open a markdown file to begin");
        });
    }
}
