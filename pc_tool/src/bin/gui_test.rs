use eframe::egui;


fn main() -> eframe::Result {
    // 0.33系では ViewportBuilder を使って窓のサイズを指定します
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0]), 
        ..Default::default()
    };

    eframe::run_native(
        "Digimatic Display v0.33",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyAppData::default()))
        }),
    )
}

#[derive(Default)]
struct MyAppData {}

impl eframe::App for MyAppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("System Standby...");
            ui.label("Waiting for data.");
        });
    }
}