//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use std::convert::TryFrom;
use std::sync::mpsc::Sender;
use std::{thread, time::Duration};

use crate::errors::{CommError, DigimaticError};
use crate::frame::{DigimaticFrame, Measurement};
use crate::logger::{MeasurementLog, RxDataLog};
use crate::sim::{frame_array_builder, generator};

// Simのループコア
pub fn run_simulation_core(
    mut receiver: Box<dyn crate::communicator::MeasurementRead>,
    mut rx_wtr: Option<csv::Writer<std::fs::File>>,
    mut m_wtr: Option<csv::Writer<std::fs::File>>,
    tx: Option<Sender<f64>>,
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
        if let Err(e) = handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &tx) {
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

/// 受信データに対する「保存・パース・送信」の共通ハンドラ
fn handle_received_data(
    data: &str,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<f64>>,
) -> Result<(), DigimaticError> {
    // 生ログの準備 (時刻はこの瞬間に固定)
    let mut rx_log = RxDataLog::new_str(data);

    // 計測データ → フレーム生成 (TryFrom chain)
    match DigimaticFrame::try_from(data).and_then(Measurement::try_from) {
        Ok(m) => {
            let val = m.to_f64();

            //  生データの保存 (Writerがあれば)
            if let Some(w) = rx_wtr {
                rx_log.save_flush(w)?;
            }

            // 測定値の保存 (Writerがあれば)
            if let Some(w) = m_wtr {
                MeasurementLog::new(val).save_flush(w)?;
            }

            // GUIへの送信 (Senderがあれば)
            if let Some(t) = tx {
                t.send(val).map_err(|_| {
                    DigimaticError::System(crate::errors::SystemError {
                        code: 99,
                        message: "Channel closed".into(),
                    })
                })?;
            }

            println!("[SIM] Decoded: {:.3} mm", val);
        }
        Err(e) => {
            // パース失敗時：エラーを載せて生ログだけは残す
            if let Some(w) = rx_wtr {
                rx_log.error_log = Some(e.clone());
                rx_log.save_flush(w)?;
            }
            eprintln!("[SIM] Parse Error: {} | Raw: {}", e, data);
        }
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

            if tx.send(hex).is_err() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(700));
        }
    });
}
