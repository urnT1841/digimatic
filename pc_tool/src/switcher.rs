//! 引数から起動モードを選択する
//! switcher.rs

use std::sync::mpsc;

use crate::communicator::CdcReceiver;
use crate::communicator::{MeasurementRead, SimReceiver};
use crate::errors::DigimaticError;
use crate::execute_communicate;
use crate::sim::execute_sim::start_geerator_thread;

#[derive(Debug)]
pub enum AppMode {
    Sim,
    Actual,
}

/// エントリポイント
pub fn run(mode: AppMode) -> Result<(), DigimaticError> {
    // 入力ソースを生成（Sim / Actual の差はここだけ）
    let input: Box<dyn MeasurementRead> = match mode {
        AppMode::Sim => {
            let (tx, rx) = mpsc::channel::<String>();

            // Sim generator起動
            start_geerator_thread(tx);

            Box::new(SimReceiver::new(rx))
        }

        AppMode::Actual => {
            // Pico探索
            let port_path = crate::communicator::wait_until_connection()
                .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

            let port = crate::communicator::open_cdc_port(&port_path, 115200)
                .map_err(DigimaticError::from)?;

            // FrameFormatは内部でStr固定（Step3方針）
            Box::new(CdcReceiver::new(
                port,
                crate::execute_communicate::FrameFormat::Str,
            ))
        }
    };

    // ★ 完全共通パイプライン
    run_pipeline(input)
}

// 引数解析
pub fn parse_args() -> Result<AppMode, DigimaticError> {
    let mut args = std::env::args();
    args.next(); // 実行パス

    let first = match args.next() {
        None => return Ok(AppMode::Sim),
        Some(s) => s.trim_start_matches('-').to_lowercase(),
    };

    match first.as_str() {
        "sim" | "s" => Ok(AppMode::Sim),
        "actual" | "a" => Ok(AppMode::Actual),
        _ => Err(DigimaticError::Argument(
            crate::errors::ArgumentError::InvalidArgs(first),
        )),
    }
}

// STEP3 core pipeline
// Sim / Actual 共通ループ

pub fn run_pipeline(mut input: Box<dyn MeasurementRead>) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(execute_communicate::create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(execute_communicate::create_log_writer("measurement.csv")?);

    loop {
        // data受信
        let data = match input.read_str_measurement() {
            Ok(d) if d.is_empty() => continue,
            Ok(d) => d,
            Err(e) => {
                if e.is_fatal() {
                    return Err(e);
                }
                eprintln!("[Pipeline] input error: {}", e);
                continue;
            }
        };

        // parse等の処理実施
        if let Err(e) =
            execute_communicate::handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &None)
        {
            if e.is_fatal() {
                return Err(e);
            }
            eprintln!("[Pipeline] processing error: {}", e);
        }
    }
}
