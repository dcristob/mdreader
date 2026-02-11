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

impl Theme {
    pub fn apply(&self, ctx: &egui::Context) {
        match self {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }
}
