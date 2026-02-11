mod app;
mod args;
mod file;
mod markdown;
mod theme;

use app::MdReaderApp;
use args::Args;

fn main() -> eframe::Result {
    let args = Args::parse();

    let initial_app = MdReaderApp::new(args.file);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Markdown Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(initial_app))),
    )
}
