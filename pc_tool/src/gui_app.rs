use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::config::{ConnectionInfo, FrameFormat, GuiConfig};
use crate::errors::DigimaticError;
use crate::frame::{Measurement, Unit};
use crate::measurement_history::MeasurementHistory;
use crate::presentation::format_with_display_unit;

struct DisplayApp {
    measurement_data: Measurement,
    receiver: Receiver<Measurement>, // 受信機を格納
    config: GuiConfig,
    connection_info: ConnectionInfo,
    history: MeasurementHistory, // 計測履歴
}

const FONT_DATA: &[u8] = include_bytes!("../assets/UDEVGothic35LG-Regular.ttf");

impl DisplayApp {
    // 初期化実施関数
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx: std::sync::mpsc::Receiver<Measurement>,
        connection_info: ConnectionInfo,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);

        Self {
            measurement_data: Measurement::dummy(), // 将来的にraw_dataの扱いが変わる見込みなので dummy() で
            receiver: rx,
            config: GuiConfig::default(),
            connection_info,
            history: MeasurementHistory::default(),
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
        // 最新のデータを受信（既存の処理）
        while let Ok(new_data) = self.receiver.try_recv() {
            self.measurement_data = new_data.clone();
            self.history.add(new_data); // 履歴へ追加
        }
        // top bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.small("Connection Status: ");
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Connected (Pico)");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("v2.1.0-clean");
                    ui.separator();

                    // (BINの色, STRの色) のペアを同時に決定
                    let (bin_color, str_color) = match self.connection_info.mode {
                        FrameFormat::Bin => (
                            egui::Color32::from_rgb(255, 165, 0),
                            egui::Color32::DARK_GRAY,
                        ), // Binモード: オレンジ / 消灯
                        FrameFormat::Str => (
                            egui::Color32::DARK_GRAY,
                            egui::Color32::from_rgb(0, 150, 255),
                        ), // Strモード: 消灯 / 青
                    };

                    // それぞれのラベルに渡す
                    ui.colored_label(bin_color, "● BIN");
                    ui.separator();
                    ui.colored_label(str_color, "● STR");
                });
            });
        });

        // bottom bar
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Unit:");
                ui.selectable_value(&mut self.config.display_unit, Unit::Mm, "mm");
                ui.selectable_value(&mut self.config.display_unit, Unit::Inch, "inch");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("📄 AutoSave: Enabled");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // 単位切り替えボタンの配置
                ui.horizontal(|ui| {
                    ui.label("Unit:");
                    // セレクトボックス風のラジオボタン。現在の設定と一致するかで判定
                    ui.selectable_value(&mut self.config.display_unit, Unit::Mm, "mm");
                    ui.selectable_value(&mut self.config.display_unit, Unit::Inch, "inch");
                });

                ui.add_space(10.0);

                // 変換した値の表示
                // 実装した get_display_value を呼び出す
                let display_val =
                    format_with_display_unit(&self.measurement_data, self.config.display_unit);

                // 特大フォントで数値を表示
                ui.label(egui::RichText::new(display_val).size(80.0).strong());

                // 単位を添える
                ui.label(format!("{:?}", self.config.display_unit));

                // 履歴表示部
                ui.add_space(30.0);
                ui.separator();
                ui.label(egui::RichText::new("履歴").strong());
                ui.add_space(10.0);

                if self.history.is_empty() {
                    ui.weak("履歴はありません（データ未受信）");
                } else {
                    // ボス特製の iter_newest() で、メモリコピーなしの高速ループ描画！
                    for (idx, meas) in self.history.iter_newest().enumerate() {
                        // 履歴の数値も、現在の画面の表示単位（mm/inch）に合わせて綺麗にフォーマット
                        let history_val = format_with_display_unit(meas, self.config.display_unit);

                        ui.horizontal(|ui| {
                            ui.add_space(20.0); // 左側に少し余白（インデント）を作る

                            if idx == 0 {
                                // 🌟 1番新しい（直前の）データは、緑色でカッコよく目立たせる！
                                ui.colored_label(
                                    egui::Color32::from_rgb(0, 255, 150),
                                    format!("直前 ➡️  {}", history_val),
                                );
                            } else {
                                // 過去のデータは落ち着いたテキストで
                                ui.label(format!("過去 [{}] :  {}", idx, history_val));
                            }
                        });
                    }
                }
            });
        });

        // 常に画面を更新（ノギスからのデータを受け取り続けるため）
        ctx.request_repaint();
    }
}

// dispatcher から呼ばれる公開エントリーポイント
// 計測値 measurement構造体と，接続情報等のConnetcionIfon構造体を渡す
pub fn launch_display(
    rx: Receiver<Measurement>,
    conn_info: ConnectionInfo,
) -> Result<(), DigimaticError> {
    gui_run(rx, conn_info)?;
    Ok(())
}

fn gui_run(
    rx: std::sync::mpsc::Receiver<Measurement>,
    conn_info: ConnectionInfo,
) -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Digimatic Data Display v0.33",
        options,
        Box::new(move |cc| Ok(Box::new(DisplayApp::new(cc, rx, conn_info)))),
    )
}
