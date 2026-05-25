//! # 起動モード・ディスパッチャモジュール
//! dispatcher.rs
//!
//! このモジュールは、CLI引数や設定（`AppConfig`）を解析し、
//! アプリケーションの適切な実行パイプラインへ処理の振り分けを行う
//!
//! ## 主な役割
//! - 実機接続モード（Actual Mode）におけるノギスデータ待ち受けパイプラインの起動
//! - シミュレータモード（Sim Mode）における仮想データ生成パイプラインの起動
//! - GUIモードとCLIモードの実行制御
//!
//! ## データフロー
//! `main` 関数から引数情報を受け取り、内部で各実行モジュール（`received_data_handler` や GUIイベントループなど）を
//! 適切に呼び出して、アプリケーションのライフサイクルをコントロールする

use std::sync::mpsc;

use crate::communicator::{CdcReceiver, MeasurementRead, SimReceiver};
use crate::config::{AppConfig, ConsoleMode, DataSource, UiMode};
use crate::errors::DigimaticError;
use crate::frame::Measurement;
use crate::received_data_handler::{create_log_writer, handle_received_data};

/// エントリポイント
pub fn run(config: AppConfig) -> Result<(), DigimaticError> {
    let input: Box<dyn MeasurementRead> = match config.source {
        DataSource::Sim => {
            // sim用チャンネル作成 -> sim thread生成 → Box詰め
            let (tx_raw, rx_raw) = mpsc::channel();
            crate::sim::execute_sim::start_generator_thread(tx_raw);
            Box::new(SimReceiver::new(rx_raw))
        }
        DataSource::Actual => {
            let port_path = crate::communicator::wait_until_connection()
                .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

            let port = crate::communicator::open_cdc_port(&port_path, 115200)?;

            Box::new(CdcReceiver::new(port, config.format))
        }
    };

    //ここにuiモード分け
    match config.ui {
        UiMode::Gui => {
            let (tx_gui, rx_gui) = mpsc::channel();
            // パイプラインを別スレッドで起動
            // inputの所有権をスレッド内に移動させる
            std::thread::spawn(move || {
                if let Err(e) = run_pipeline(input, Some(tx_gui), config.console_mode) {
                    eprintln!("[Error] Pipeline failed: {:?}", e);
                }
            });
            // メインスレッドでGUIを起動（rx_guiからデータ受け取れる)
            crate::gui_app::launch_display(rx_gui).map_err(DigimaticError::from)
        }
        UiMode::Cli => {
            // cliの時はメインスレッドで直接パイプラン実行
            // txは不要 → Noneにしておく
            run_pipeline(input, None, config.console_mode)
        }
    }
}

// 共通ループ
fn run_pipeline(
    mut input: Box<dyn MeasurementRead>,
    tx: Option<mpsc::Sender<Measurement>>,
    console_mode: ConsoleMode,
) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(create_log_writer("measurement.csv")?);

    let frame_mode = input.get_format();
    loop {
        // data受信
        // read_measurement は measurement構造体を返すので異常値は来ない
        let data = input.read_measurement()?;

        // 共通ハンドラ処理
        handle_received_data(
            &data,
            &mut rx_wtr,
            &mut m_wtr,
            &tx,
            frame_mode,
            console_mode,
        )?;
    }
}
