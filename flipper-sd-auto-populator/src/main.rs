mod app;
mod copier;
mod drive;
mod profile;

use app::MyApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 640.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Flipper SD Provisioner",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
