//! # Graphical User Interface (GUI) Application Module
//!
//! `gui_app.rs`
//!
//! This module builds the immediate-mode visual interface powered by the `egui` / `eframe` ecosystem.
//! It acts as the final terminal consumer in the data pipeline, polling asynchronously from
//! the background MPSC channel to drive realtime gauge projection plots and history logs.
//!
//! ## UI Hierarchy Matrix
//! ```text
//! DisplayApp (Global State Manager)
//!  ├── setup_custom_fonts (Invoked once at initialization: Injects TTF asset)
//!  │
//!  └── update (Invoked every frame: Main rendering tick lifecycle)
//!        ├── MPSC Stream Ingestion (Drains the receiver queue completely)
//!        ├── draw_top_bar (Renders attachment link layer status and frame protocols)
//!        ├── draw_under_bar (Renders quick access options)
//!        └── CentralPanel (Core body context)
//!              └── vertical_centered
//!                    ├── draw_main_measurement (Oversized primary telemetry display)
//!                    └── draw_main_history (Scroll-protected chronological table logs)
//! ```
//! GUI構成
//! ```text
//! DisplayApp (アプリの親玉：状態の管理)
//!  ├── setup_custom_fonts (起動時に1回だけ：フォントの仕込み)
//!  │
//!  └── update (毎フレーム走る描画の司令)
//!        ├── MPSCデータ受信 (ロジック)
//!        ├── draw_top_bar (接続ステータスやモード表示)
//!        ├── draw_under_bar (下部の単位切り替えやステータス)
//!        └── CentralPanel (メイン画面)
//!              └── vertical_centered
//!                    ├── draw_main_measurement (特大の現在の計測値)
//!                    └── draw_main_history (過去データの履歴リスト)
//! ```

use eframe::egui;
use egui::Color32;
use egui_plot::{Line, Plot, PlotPoints};
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

    // top bar
    fn draw_top_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.small("Connection Status: ");
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "Connected (Pico)");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small("v2.1.0-clean");
                    ui.separator();

                    let (bin_color, str_color) = match self.connection_info.mode {
                        FrameFormat::Bin => (
                            egui::Color32::from_rgb(255, 165, 0),
                            egui::Color32::DARK_GRAY,
                        ),
                        FrameFormat::Str => (
                            egui::Color32::DARK_GRAY,
                            egui::Color32::from_rgb(0, 150, 255),
                        ),
                    };

                    ui.colored_label(bin_color, "● BIN");
                    ui.separator();
                    ui.colored_label(str_color, "● STR");
                });
            });
        });
    }

    // bottom bar
    fn draw_under_bar(&mut self, ctx: &egui::Context) {
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
    }

    // main (計測データ)
    fn draw_main_measurement(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);

        // 単位切り替えボタンの配置
        ui.horizontal(|ui| {
            ui.label("Unit:");
            ui.selectable_value(&mut self.config.display_unit, Unit::Mm, "mm");
            ui.selectable_value(&mut self.config.display_unit, Unit::Inch, "inch");
        });

        ui.add_space(10.0);

        // 変換した値の表示
        let display_val =
            format_with_display_unit(&self.measurement_data, self.config.display_unit);

        // 特大フォントで数値を表示
        ui.label(egui::RichText::new(display_val).size(80.0).strong());

        // 単位を添える
        ui.label(format!("{:?}", self.config.display_unit));
    }

    // main 履歴
    fn draw_main_history(&self, ui: &mut egui::Ui) {
        ui.add_space(30.0);
        ui.separator();
        ui.label(egui::RichText::new(" 履歴 (最大50件)").strong());
        ui.add_space(10.0);

        if self.history.is_empty() {
            ui.weak("履歴はありません（データ未受信）");
        } else {
            // ここから下の要素を縦方向（vertical）のスクロール領域にする！
            // max_height を指定して、画面全体のレイアウトが崩れないようにガード
            egui::ScrollArea::vertical()
                .max_height(200.0) // お好みの高さ（ピクセル）で固定
                .show(ui, |ui| {
                    // この中身は今までのボスのコードと100%同じでOK！
                    for (idx, meas) in self.history.iter_newest().enumerate() {
                        let history_val = format_with_display_unit(meas, self.config.display_unit);

                        ui.horizontal(|ui| {
                            ui.add_space(20.0);

                            if idx == 0 {
                                ui.colored_label(
                                    egui::Color32::from_rgb(0, 255, 150),
                                    format!("現測定値 ➡️  {history_val}"),
                                );
                            } else {
                                ui.label(format!("過去 [{idx}] :  {history_val}"));
                            }
                        });
                    }
                }); // スクロールエリアここまで
        }
    }

    // 履歴グラフ
    fn draw_main_plot(&self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.separator();
        ui.label(egui::RichText::new("リアルタイムトレンドグラフ").strong());
        ui.add_space(10.0);

        if self.history.is_empty() {
            ui.weak("グラフデータなし");
            return;
        }

        // 履歴を古い順に並び変えてプロット用データへ
        //   x axis: index(old -> new)  ,  y axis: mes data
        let points: PlotPoints = self
            .history
            .iter_newest()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .enumerate()
            .map(|(idx, meas)| {
                let val = meas.to_f64();
                [idx as f64, val]
            })
            .collect();

        let line = Line::new(points)
            .name("Measurement value")
            .color(Color32::from_rgb(0, 255, 150))
            .width(2.0);

        Plot::new("measurement_trend")
            .view_aspect(3.0)
            .show_grid(true)
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }
}

impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // MPSCキューから最新のデータを全て引っこ抜く（バックエンド通信の消化）
        while let Ok(new_data) = self.receiver.try_recv() {
            self.measurement_data = new_data;
            self.history.add(new_data);
        }

        // 各コンポーネントを呼び出す
        self.draw_top_bar(ctx);
        self.draw_under_bar(ctx);

        // 中央のメインパネル
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                self.draw_main_measurement(ui); // 計測表示
                self.draw_main_plot(ui);
                self.draw_main_history(ui); // 履歴表示
            });
        });

        ctx.request_repaint();
    }
}

// dispatcher から呼ばれる公開エントリーポイント
// 計測値 measurement構造体と，接続情報等のConnetcionIfon構造体を渡す
/// Launches the native display interface window block.
/// This acts as a blocking terminal call that runs until the window frame is closed by the user.
///
/// # Errors
///
/// This function bubbles up an underlying [`DigimaticError`] wrapper variant if `eframe` fails
/// to bind native graphics context resources or hook system OS window loops.
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
