mod app;
pub use app::MatahatanApp;

pub fn show_maze() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "MatahatanApp",
        native_options,
        Box::new(|cc| Box::new(app::MatahatanApp::new(cc))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
}
