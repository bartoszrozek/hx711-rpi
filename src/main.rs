use eframe::egui;
use std::path::PathBuf;

struct Vessel {
    name: &'static str,
    filename: &'static str,
    weight: f32,
}

fn vessel_image_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("images")
        .join(filename)
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Weight",
        options,
        Box::new(|_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}

struct MyApp {
    vessels: Vec<Vessel>,
    current_index: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            vessels: vec![
                Vessel {
                    name: "Pan A",
                    filename: "pan1.png",
                    weight: 1.23,
                },
                Vessel {
                    name: "Pan B",
                    filename: "pan2.png",
                    weight: 2.75,
                },
                Vessel {
                    name: "Pan C",
                    filename: "pan3.png",
                    weight: 0.98,
                },
            ],
            current_index: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    if self.vessels.is_empty() {
                        ui.heading("No vessels configured");
                        return;
                    }

                    let vessel = &self.vessels[self.current_index];

                    ui.vertical_centered(|ui| {
                        ui.heading(format!("Vessel: {}", vessel.name));
                        ui.label(format!("Weight: {:.2} kg", vessel.weight));
                    });
                    // ui.add_space(300.0);
                    ui.horizontal(|ui| {
                        let row_width = 80.0 + 100.0 + 300.0 + 100.0 + 80.0;
                        let left_margin = ((ui.available_width() - row_width) * 0.5).max(0.0);
                        ui.add_space(left_margin);

                        if ui
                            .add_sized([80.0, 30.0], egui::Button::new("Left"))
                            .clicked()
                        {
                            if self.current_index == 0 {
                                self.current_index = self.vessels.len() - 1;
                            } else {
                                self.current_index -= 1;
                            }
                        };
                        ui.add_space(100.0);
                        let image_path = vessel_image_path(vessel.filename);
                        let image = match std::fs::read(&image_path) {
                            Ok(bytes) => egui::Image::from_bytes(
                                format!("bytes://{}", vessel.filename),
                                bytes,
                            ),
                            Err(_) => {
                                egui::Image::new(egui::include_image!("../assets/images/pan1.png"))
                            }
                        };
                        ui.add(
                            image
                                .fit_to_exact_size(egui::vec2(300.0, 300.0))
                                .maintain_aspect_ratio(true),
                        );
                        ui.add_space(100.0);
                        if ui
                            .add_sized([80.0, 30.0], egui::Button::new("Right"))
                            .clicked()
                        {
                            self.current_index = (self.current_index + 1) % self.vessels.len();
                        };
                    });
                    // ui.add_space(100.0);
                },
            );
        });
    }
}
