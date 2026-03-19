use eframe::egui;
use rand::Rng;

fn main() -> eframe::Result {
    // 0.33系では ViewportBuilder を使って窓サイズを指定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Digimatic Display v0.33",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "my_font".to_owned(),
                // .into() を足すことで FontData -> Arc<FontData> に変換
                egui::FontData::from_static(include_bytes!(
                    "../../assets/UDEVGothic35LG-Regular.ttf"
                ))
                .into(),
            );
            // 3. プロポーショナルフォント（通常の文字）の最優先に設定
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "my_font".to_owned());

            // 4. コンテキストに反映
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(MyAppData::default()))
        }),
    )
}

fn big_num_dice() -> u32 {
    let mut rng = rand::rng();
    let dice = rng.random_range(1..=150_00);
    dice
}

#[derive(Default)]
struct MyAppData {
    dice_value: u32,
}

impl MyAppData {
    // 外部とのインターフェイス
    pub fn set_dice_value(&mut self, new_value: u32) {
        self.dice_value = new_value;
    }
}

impl eframe::App for MyAppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 中央揃えにすると「デカ窓」っぽくなります
                ui.add_space(20.0);
                ui.heading("GUI case study");
                ui.add_space(30.0);

                // --- 表示エリア ---
                let display_val = self.dice_value as f64 / 100.0;
                let text = egui::RichText::new(format!("{:.2} mm", display_val))
                    .size(100.0) // ここでデカく！
                    .strong()
                    .color(egui::Color32::from_rgb(255, 165, 0)); // オレンジ色で目立たせる

                ui.label(text);

                ui.add_space(30.0);

                // --- 操作エリア ---
                // ボタンが押されたら true を返すので、そのままif文で処理
                if ui
                    .add(egui::Button::new("サイコロを振る").min_size(egui::vec2(200.0, 50.0)))
                    .clicked()
                {
                    // 自作のサイコロ関数を呼ぶ
                    let result = big_num_dice();
                    // 構造体のメソッドを通じて値をセット
                    self.set_dice_value(result);
                }
            });
        });
    }
}
