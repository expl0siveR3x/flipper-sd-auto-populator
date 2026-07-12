use eframe::egui;

pub enum AppScreen {
    Main,
    //CopyInProgress
    //ProfileEditor,
}
pub struct MyApp {
    pub screen: AppScreen,
    //pub profiles: Vec<Profile>,
    //pub selected_profile: usize,
    //pub detected_drives: Vec<DriveInfo>,
    //pub selected_drive: Option<std::path::PathBuf>,
    // job_rx, job_log, etc. added later
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        //let profiles = profile::load_profiles("profiles.json").unwrap_or_default();
        Self {
            screen: AppScreen::Main,
            //profiles,
            //selected_profile: 0,
            //detected_drives: vec![],
            //selected_drive: None,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            match self.screen {
                AppScreen::Main => self.show_main(ui),
                //AppScreen::ProfileEditor => self.show_editor(ui),
            }
        });
    }
}

impl MyApp {
    fn show_main(&mut self, ui: &mut egui::Ui) {
        // widget code for the main screen
        let mut my_string = String::from("TEST");
        let mut my_bool = false;
        ui.label("text"); // static text
        ui.heading("Title"); // bigger label
        ui.text_edit_singleline(&mut my_string); // one-line editable text
        ui.text_edit_multiline(&mut my_string); // multi-line
        ui.button("Click me").clicked(); // returns bool this frame
        ui.checkbox(&mut my_bool, "Enable X");
        ui.selectable_label(my_bool, "Tab A").clicked(); // tab-like toggle
        ui.colored_label(egui::Color32::RED, "Error!");
        ui.hyperlink_to(
            "Momentum Firmware Updater",
            "https://momentum-fw.dev/update",
        );
        ui.hyperlink_to(
            "Momentum Asset Pack Installer",
            "https://momentum-fw.dev/asset-packs",
        );
    }
    fn show_editor(&mut self, ui: &mut egui::Ui) {
        // widget code for the profile editor
    }
}
