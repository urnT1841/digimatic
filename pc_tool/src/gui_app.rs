use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::errors::DigimaticError;
use crate::frame::{Measurement, Unit};
use crate::presentation::format_measurement_value_with_unit;
//設定用構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiConfig {
    pub display_unit: Unit,
    pub font_size: f32,
    pub dark_mode: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            display_unit: Unit::Mm,
            font_size: 24.0,
            dark_mode: true,
        }
    }
}

struct DisplayApp {
    measurement_data: Measurement,
    receiver: Receiver<Measurement>, // 受信機を格納
    config: GuiConfig,
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
            config: GuiConfig::default(),
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

    // 単位変換
    fn get_display_value(&self, target_unit: Unit) -> f64 {
        const INCH2MM: f64 = 25.4;
        let m = &self.measurement_data;
        // 変換用に計測値用意
        let base_value = m.to_f64();
        match (m.unit, target_unit) {
            (Unit::Mm, Unit::Inch) => base_value / INCH2MM,
            (Unit::Inch, Unit::Mm) => base_value * INCH2MM,
            _ => base_value, // 変わらない場合
        }
    }
}

impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 最新のデータを受信（既存の処理）
        if let Ok(new_data) = self.receiver.try_recv() {
            self.measurement_data = new_data;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // --- 2. 単位切り替えボタンの配置 ---
                ui.horizontal(|ui| {
                    ui.label("Unit:");
                    // セレクトボックス風のラジオボタン。現在の設定と一致するかで判定
                    ui.selectable_value(&mut self.config.display_unit, Unit::Mm, "mm");
                    ui.selectable_value(&mut self.config.display_unit, Unit::Inch, "inch");
                });

                ui.add_space(10.0);

                // --- 3. 変換した値の表示 ---
                // 実装した get_display_value を呼び出す
                let display_val = self.get_display_value(self.config.display_unit);

                // インチの場合は小数点4桁、mmの場合は2桁など、単位で書式を変えるとプロっぽい
                let format_str = if self.config.display_unit == Unit::Inch {
                    format!("{:.4}", display_val)
                } else {
                    format!("{:.2}", display_val)
                };

                // 特大フォントで数値を表示
                ui.label(egui::RichText::new(format_str).size(80.0).strong());

                // 単位を添える
                ui.label(format!("{:?}", self.config.display_unit));
            });
        });

        // 常に画面を更新（ノギスからのデータを受け取り続けるため）
        ctx.request_repaint();
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
