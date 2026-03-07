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
    // Try to load the icon from the project directory
    let icon_path = std::path::Path::new("markdown.png");
    
    if let Ok(icon_data) = std::fs::read(icon_path) {
        if let Ok(image) = image::load_from_memory(&icon_data) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }
    }
    
    // Fallback: try to load from the executable directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let icon_path = exe_dir.join("markdown.png");
            if let Ok(icon_data) = std::fs::read(icon_path) {
                if let Ok(image) = image::load_from_memory(&icon_data) {
                    let rgba = image.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    
                    return Some(egui::IconData {
                        rgba: rgba.into_raw(),
                        width,
                        height,
                    });
                }
            }
        }
    }
    
    None
}

fn main() -> eframe::Result {
    let args = Args::parse();

    let initial_app = MdReaderApp::new(args.file);

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 700.0])
        .with_decorations(true)
        .with_transparent(false);
    
    // Set the window icon if available
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
