use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

static FONTS_LOADED: AtomicBool = AtomicBool::new(false);

impl Theme {
    pub fn apply(&self, ctx: &egui::Context) {
        // Load custom fonts once (apply() is called every frame in immediate mode)
        if !FONTS_LOADED.swap(true, Ordering::Relaxed) {
            use egui::epaint::text::{FontInsert, InsertFontFamily, FontPriority};
            use egui::FontData;

            ctx.add_font(FontInsert::new(
                "inter",
                FontData::from_static(include_bytes!("../fonts/Inter-Regular.ttf")),
                vec![InsertFontFamily {
                    family: egui::FontFamily::Proportional,
                    priority: FontPriority::Highest,
                }],
            ));

            ctx.add_font(FontInsert::new(
                "jetbrains_mono",
                FontData::from_static(include_bytes!("../fonts/JetBrainsMono-Regular.ttf")),
                vec![InsertFontFamily {
                    family: egui::FontFamily::Monospace,
                    priority: FontPriority::Highest,
                }],
            ));
        }

        // Apply theme visuals
        match self {
            Theme::Light => {
                let mut visuals = egui::Visuals::light();
                visuals.window_fill = egui::Color32::from_gray(250);
                ctx.set_visuals(visuals);
            }
            Theme::Dark => {
                let mut visuals = egui::Visuals::dark();
                visuals.window_fill = egui::Color32::from_gray(30);
                ctx.set_visuals(visuals);
            }
        }

        // Set text style sizes and global spacing
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(30.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(17.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(15.0, egui::FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        ctx.set_style(style);
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }
}
