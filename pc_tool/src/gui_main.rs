use std::sync::mpsc::Receiver;
use eframe::egui;

use digimatic::sim::generator::generator;

struct DisplayApp {
    measurement_data: f64,
    receiver: Receiver<f64>, // 受信機を格納
}

impl DisplayApp {
    // 起動時に受信機を受け取るための専用の初期化関数
    fn with_receiver(rx: Receiver<f64>) -> Self {
        Self {
            measurement_data: 0.0,
            receiver: rx,
        }
    }
}
impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // データ受信チェック
        // try_recv() は「データがあれば取る、なければすぐ次へ」という動き
        // while を使うことで、溜まっているデータをすべて処理する
        while let Ok(new_val) = self.receiver.try_recv() {
            self.measurement_data = new_val;
            ctx.request_repaint();
        }

        // 描画
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("計測中").size(20.0));

                let val_mm = self.measurement_data as f64 / 100.0;
                ui.label(
                    egui::RichText::new(format!("{:.2} mm", val_mm))
                        .size(120.0)
                        .strong(),
                );
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Digimatic Data Display v0.33",
        options,
        Box::new(|cc| {
            // 送受信窓口
            let (tx, rx) = std::sync::mpsc::channel::<f64>();
            let ctx_for_thread = cc.egui_ctx.clone();

            // スレッド起動
            std::thread::spawn(move || {
                loop {
                    let val = generator();
                    if tx.send(val).is_err() {
                         break;
                    }
                    ctx_for_thread.request_repaint();
                    std::thread::sleep(std::time::Duration::from_millis(1200));
                }
            });

            // フォントの設定
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\msmincho.ttc")).into(),
            );
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "my_font".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            // rxを送って窓開始
            Ok(Box::new(DisplayApp::with_receiver(rx)))
        }),
    )
}
