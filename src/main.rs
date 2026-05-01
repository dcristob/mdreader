mod app;
mod args;
mod config;
mod file;
mod render;
mod search;
mod theme;

use app::MdReaderApp;
use args::Args;

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../markdown.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result {
    let args = Args::parse();

    let initial_app = MdReaderApp::new(args.file);

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 700.0])
        .with_decorations(true)
        .with_transparent(false)
        .with_app_id("mdreader");

    if let Some(icon) = load_icon() {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Markdown Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(initial_app))),
    )
}
