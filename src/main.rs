use eframe::egui;

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

#[derive(Default)]
struct MyApp {
    weight: f32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                ui.available_size(),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(format!("Weight: {:.2} kg", self.weight));

                        if ui.button("Refresh").clicked() {
                            self.weight = 1.23;
                        }
                    });
                    // ui.add_space(300.0);
                    ui.horizontal(|ui| {
                        ui.add_space(100.0);

                        if ui
                            .add_sized([80.0, 30.0], egui::Button::new("Left"))
                            .clicked()
                        {
                            print!("left")
                        };
                        ui.add_space(100.0);
                        ui.add(
                            egui::Image::new(egui::include_image!("../assets/images/pan1.png"))
                                .fit_to_exact_size(egui::vec2(300.0, 200.0)),
                        );
                        ui.add_space(100.0);
                        if ui
                            .add_sized([80.0, 30.0], egui::Button::new("Right"))
                            .clicked()
                        {
                            print!("right")
                        };
                    });
                    // ui.add_space(100.0);
                },
            );
        });
    }
}
