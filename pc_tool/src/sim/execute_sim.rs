//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use std::convert::TryFrom;
use std::sync::mpsc::Sender;
use std::{thread, time::Duration};

use crate::errors::{CommError, DigimaticError};
use crate::execute_communicate;
use crate::frame::Measurement;
use crate::sim::{frame_array_builder, generator};

// Simのループコア
pub fn run_simulation_core(
    mut receiver: Box<dyn crate::communicator::MeasurementRead>,
    mut rx_wtr: Option<csv::Writer<std::fs::File>>,
    mut m_wtr: Option<csv::Writer<std::fs::File>>,
    tx: Option<Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    const WAIT_TIME_MS: u64 = 700;

    loop {
        // 受信 (Timeoutは無視し、それ以外のエラーは上位へ)
        let data = match receiver.read_str_measurement() {
            Ok(d) if d.is_empty() => continue,
            Ok(d) => d,
            // タイムアウト（非致命的）は無視して次へ
            Err(DigimaticError::Comm(CommError::Timeout)) => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            // timeout以外
            Err(e) => {
                if e.is_fatal() {
                    return Err(e); // 致命的なら上位へ報告
                }
                eprintln!("Non-fatal sim error: {}", e);
                continue;
            }
        };

        // データのパイプライン処理
        if let Err(e) =
            execute_communicate::handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &tx)
        {
            // Channel閉鎖など、ループを止めるべき致命的エラーなら抜ける
            match e {
                DigimaticError::Comm(CommError::ConnectionClosed) => break,
                _ => {
                    eprintln!("Processing error: {}", e)
                }
            }
        }
        thread::sleep(Duration::from_millis(WAIT_TIME_MS));
    }
    Ok(())
}

/// データ生成スレッド
/// channel使ってreceiverに流し込む
pub fn start_geerator_thread(tx: Sender<String>) {
    std::thread::spawn(move || {
        loop {
            let val = generator::generator();
            let frame = frame_array_builder::build_frame_array(val);
            let hex: String = frame.iter().map(|b| format!("{:X}", b)).collect();

            // デコード前データ
            println!("[SIM] gen = {:.3} -> frame={:?}, HEX={:?}", val, frame, hex);

            if tx.send(hex).is_err() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(700));
        }
    });
}
