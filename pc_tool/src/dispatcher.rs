//! dispatcher.rs
//!
//! # Pipeline Dispatcher Module
//!
//! This module orchestrates the initialization and lifecycle of execution pipelines
//! based on the parsed global `AppConfig`. It evaluates the ingestion sources and
//! UI presentation modes to securely branch execution threads.
//!
//! ## Key Responsibilities
//! - Spawning virtual data generator threads when running in `DataSource::Sim` mode.
//! - Resolving physical serial paths and opening link layers when in `DataSource::Actual` mode.
//! - Directing data flow paths via multi-producer, single-consumer (MPSC) channels to GUI or CLI sinks.
//!
//! ## Data Flow Topology
//! It consumes the configuration block from the entry point, dynamically wraps the data reader
//! into a trait object (`Box<dyn MeasurementRead>`), and establishes the infinite processing loop.

use std::sync::mpsc;

use crate::communicator::{CdcReceiver, MeasurementRead, SimReceiver};
use crate::config::{AppConfig, ConnectionInfo, ConsoleMode, DataSource, UiMode};
use crate::errors::DigimaticError;
use crate::frame::Measurement;
use crate::received_data_handler::{create_log_writer, handle_received_data};
use crate::sim::execute_sim::FrameGenerator;

/// エントリポイント
pub fn run(config: AppConfig) -> Result<(), DigimaticError> {
    let input: Box<dyn MeasurementRead> = match config.source {
        DataSource::Sim => {
            // sim用チャンネル作成 -> sim thread生成 → Box詰め
            let (tx_raw, rx_raw) = mpsc::channel();
            // TODO ここでの generator呼び出しは見直す必要あり
            FrameGenerator::new(tx_raw, config.format, config.sim_mode).start_generator_thred();
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
                    eprintln!("[Error] Pipeline failed: {e:?}");
                }
            });
            // メインスレッドでGUIを起動（rx_guiからデータ受け取れる)
            let conn_info = ConnectionInfo::new(config.format);
            crate::gui_app::launch_display(rx_gui, conn_info)
        }
        UiMode::Cli => {
            // cliの時はメインスレッドで直接パイプラン実行
            // txは不要 → Noneにしておく
            run_pipeline(input, None, config.console_mode)
        }
    }
}

fn run_pipeline(
    mut input: Box<dyn MeasurementRead>,
    tx: Option<mpsc::Sender<Measurement>>,
    console_mode: ConsoleMode,
) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(create_log_writer("measurement.csv")?);

    loop {
        // data受信
        // read_measurement は measurement構造体を返すので異常値は来ない
        let data = input.read_measurement()?;
        handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &tx, console_mode)?;
    }
}
