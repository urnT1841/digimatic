//! 引数から起動モードを選択する
//! switcher.rs

use std::sync::mpsc;

use crate::args::{AppConfig, DataSource, UiMode};
use crate::communicator::{CdcReceiver, MeasurementRead, SimReceiver};
use crate::errors::DigimaticError;
use crate::execute_communicate;
use crate::execute_communicate::handle_received_data;
use crate::frame::Measurement;

/// エントリポイント
pub fn run(config: AppConfig) -> Result<(), DigimaticError> {
    let input: Box<dyn MeasurementRead> = match config.source {
        DataSource::Sim => {
            // sim用チャンネル作成 -> sim thred生成 → Box詰め
            let (tx_raw, rx_raw) = mpsc::channel();
            crate::sim::execute_sim::start_geerator_thread(tx_raw);
            Box::new(SimReceiver::new(rx_raw))
        }
        DataSource::Actual => {
            let port_path = crate::communicator::wait_until_connection()
                .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

            let port = crate::communicator::open_cdc_port(&port_path, 115200)?;

            Box::new(CdcReceiver::new(
                port,
                crate::execute_communicate::FrameFormat::Str,
            ))
        }
    };

    //ここにuiモード分け
    match config.ui {
        UiMode::Gui => {
            let (tx_gui, rx_gui) = mpsc::channel();
            // パイプラインを別スレッドで起動
            // inputの所有権をスレッド内に移動させる
            std::thread::spawn(move || {
                if let Err(e) = run_pipeline(input, Some(tx_gui)) {
                    eprintln!("[Error] Pipeline failde: {:?}", e);
                }
            });
            // メインスレッドでGUIを起動（rx_guiからデータ受け取れる)
            crate::gui_app::launch_display(rx_gui).map_err(DigimaticError::from)
        }
        UiMode::Cli => {
            // cliの時はメインスレッドで直接パイプラン実行
            // txは不要 → Noneにしておく
            run_pipeline(input, None)
        }
    }
}

// 共通ループ
pub fn run_pipeline(
    mut input: Box<dyn MeasurementRead>,
    tx: Option<mpsc::Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(execute_communicate::create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(execute_communicate::create_log_writer("measurement.csv")?);

    let frame_mode = input.get_format();
    loop {
        // data受信
        // read_measurement は measurement構造体を返すので異常値は来ない
        let data = input.read_measurement()?;

        // 共通ハンドラ処理
        handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &tx, frame_mode)?;

        if tx.is_none() {
            // cli modeの時のコンソールへの表示など。下記はダミー
            print!("実行中");
        }
    }
}
