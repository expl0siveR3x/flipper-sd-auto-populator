mod app;
mod copier;
mod drive;
mod profile;

use app::MyApp;

//TODO
//Backup sd before doing operations to it as an optional mode (good for modifying customer's flippers)

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Flipper SD Provisioner",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
