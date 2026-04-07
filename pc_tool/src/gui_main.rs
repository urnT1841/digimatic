use eframe::egui;
use std::sync::mpsc::Receiver;

struct DisplayApp {
    measurement_data: f64,
    receiver: Receiver<f64>, // 受信機を格納
}

const FONT_DATA: &[u8] = include_bytes!("../assets/UDEVGothic35LG-Regular.ttf");

impl DisplayApp {
    // 初期化実施関数
    pub fn new(cc: &eframe::CreationContext<'_>, rx: std::sync::mpsc::Receiver<f64>) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);

        Self {
            measurement_data: 0.0,
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

                // 本番ノギスデータは最終状態で送られてくるのでそのまま
                let val_mm = self.measurement_data;

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

            // スレッド起動
            std::thread::spawn(move || {
                loop {
                    let tx_clone = tx.clone();
                    // let val = generator();
                    // if tx.send(val).is_err() {
                    //     break;
                    // }
                    // ノギスと接続させて表示(本番)
                    if let Err(e) = digimatic::execute_communicate::run_actual_loop(tx_clone) {
                        eprintln!("通信エラー： {} ", e);
                    }
                }
            });
            // rxを送って窓開始
            Ok(Box::new(DisplayApp::new(cc, rx)))
        }),
    )
}
