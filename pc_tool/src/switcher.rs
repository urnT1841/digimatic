//! 引数から起動モードを選択する
//! switcher.rs

use std::sync::mpsc;
use std::sync::mpsc::Sender;

use crate::communicator::{SimReceiver, MeasurementRead};
use crate::errors::DigimaticError;
use crate::execute_communicate;
use crate::frame::Measurement;
use crate::sim::execute_sim::{run_simulation_core, start_geerator_thread};

#[derive(Debug)]
pub enum AppMode {
    Sim,
    Actual,
}

/// エントリポイント
pub fn run(mode: AppMode) -> Result<(), DigimaticError> {
    match mode {
        AppMode::Sim => run_sim(),
        AppMode::Actual => run_actual(),
    }
}

// Sim mode (CLI/GUI兼用)
fn run_sim() -> Result<(), DigimaticError> {
    let (tx_raw, rx_raw) = mpsc::channel::<String>();

    // データ生成スレッド起動
    start_geerator_thread(tx_raw);

    // SimReceiverを入力源にする
    let sim_receiver = SimReceiver::new(rx_raw);

    // ★ 共通パイプラインへ
    run_pipeline(Box::new(sim_receiver))
}


// Actual mode  (Pico接続)
fn run_actual() -> Result<(), DigimaticError> {
    let (tx, _rx) = mpsc::channel::<Measurement>();

    execute_communicate::run_actual_loop(tx)
}

// 引数解析
pub fn parse_args() -> Result<AppMode, DigimaticError> {
    let mut args = std::env::args();
    args.next(); // 実行パススキップ

    let first_arg = match args.next() {
        None => return Ok(AppMode::Sim), // デフォルトはSim（安全側）
        Some(s) => s.trim_start_matches('-').to_lowercase(),
    };

    match first_arg.as_str() {
        "sim" | "s" => Ok(AppMode::Sim),
        "actual" | "a" => Ok(AppMode::Actual),
        _ => Err(DigimaticError::Argument(
            crate::errors::ArgumentError::InvalidArgs(first_arg),
        )),
    }
}


// STEP3 core pipeline
// Sim / Actual 共通ループ

pub fn run_pipeline(
    mut input: Box<dyn MeasurementRead>,
) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(execute_communicate::create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(execute_communicate::create_log_writer("measurement.csv")?);

    loop {
        // データ受信
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

        // パース等の処理実施
        if let Err(e) = execute_communicate::handle_received_data(
            &data,
            &mut rx_wtr,
            &mut m_wtr,
            &None,
        ) {
            if e.is_fatal() {
                return Err(e);
            }
            eprintln!("[Pipeline] processing error: {}", e);
        }
    }
}