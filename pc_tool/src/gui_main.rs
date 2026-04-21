use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::{errors::DigimaticError, frame::Measurement};

struct DisplayApp {
    measurement_data: Measurement,
    receiver: Receiver<Measurement>, // 受信機を格納
}

// mm表示用変換
impl Measurement {
    fn display_mm(&self) -> String {
        format!("{:.2} mm", self.to_f64())
    }
}

const FONT_DATA: &[u8] = include_bytes!("../assets/UDEVGothic35LG-Regular.ttf");

impl DisplayApp {
    // 初期化実施関数
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: std::sync::mpsc::Receiver<Measurement>,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);

        Self {
            measurement_data: Measurement::dummy(), // 将来にraw_dataの扱いが変わる見込みなので dummy() で
            receiver: rx,
        }
    }

    // font設定
    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "digital_num".to_owned(),
            egui::FontData::from_static(FONT_DATA).into(),
            //egui::FontData::from_static(include_bytes!("../assets/IBMPlexSansJP-SemiBold.ttf")).into(),
        );

        // 数字をこのフォントで出すために、全ファミリーの最優先に設定
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace].iter_mut() {
            fonts
                .families
                .get_mut(family)
                .unwrap()
                .insert(0, "digital_num".to_owned());
        }
        ctx.set_fonts(fonts);
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

                ui.label(
                    egui::RichText::new(self.measurement_data.display_mm())
                        .size(120.0)
                        .strong(),
                );
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

// switcher から呼ばれる公開エントリポイント
pub fn launch_display(rx: Receiver<Measurement>) -> Result<(), DigimaticError> {
    gui_run(rx)?;
    Ok(())
}

fn gui_run(rx: std::sync::mpsc::Receiver<Measurement>) -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Digimatic Data Display v0.33",
        options,
        Box::new(move |cc| Ok(Box::new(DisplayApp::new(cc, rx)))),
    )
}
