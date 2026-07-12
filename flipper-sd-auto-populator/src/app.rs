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
    pub selected_drive: Option<std::path::PathBuf>,
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
            selected_drive: None,
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

        ui.horizontal(|ui| {
            ui.heading("Flipper SD Auto Populator BETA");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing();
                if ui.button("Edit Profiles").clicked() {};

                ui.separator();

                ui.hyperlink_to(
                    "Momentum Firmware Updater",
                    "https://momentum-fw.dev/update",
                );
                ui.hyperlink_to(
                    "Momentum Asset Pack Installer",
                    "https://momentum-fw.dev/asset-packs",
                );
            });
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("Profile:");
            egui::ComboBox::from_id_salt("profile_select")
            /*.selected_text(&self.profiles[self.selected_profile].name)
            .show_ui(ui, |ui| {
                for (i, p) in self.profiles.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_profile, i, &p.name);
                }
            });
            */
        });

        ui.add_space(8.0);
        ui.label("Target SD Card:");
        match &self.selected_drive {
            Some(path) => {
                ui.label(format!("● {}", path.display()));
            }
            None => {
                ui.colored_label(egui::Color32::YELLOW, "No drive selected");
            }
        }
        if ui.button("Choose Drive Manually...").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                self.selected_drive = Some(folder);
            }
        }

        ui.separator();
        ui.add_space(8.0);

        ui.vertical_centered(|ui| {
            let can_start = self.selected_drive.is_some();
            if ui
                .add_enabled(can_start, egui::Button::new("Move Files"))
                .clicked()
            {
                //self.start_copy();
            }
        });

        ui.add_space(8.0);
        ui.label(String::from("STATUS STRING TEST"));
    }
    fn show_editor(&mut self, ui: &mut egui::Ui) {
        // widget code for the profile editor
    }
}
