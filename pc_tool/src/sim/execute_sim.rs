//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use chrono::Local;
use csv::{Writer, WriterBuilder};
use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::{thread, time::Duration};

use crate::communicator::CdcReceiver;
use crate::frame::{DigimaticFrame, Measurement};
use crate::logger::*;
use crate::sim::frame_array_builder::build_frame_array;
use crate::sim::generator::generator;
use crate::sim::port_prepare::port_prepare;
use crate::sim::sender::{SendMode, send};

pub fn run_simmulation_loop() -> Result<(), Box<dyn std::error::Error>> {
    // ポート準備
    let mut ports = port_prepare()?;

    // 受信用構造体
    // port所有権が rx_receiverへ移る
    let mut rx_receiver = CdcReceiver::new(ports.rx);

    // 保存用にライター準備
    let mut rx_wtr = create_log_writer("rx_log.csv")?;
    let mut m_wtr = create_log_writer("measurement.csv")?;

    const WATI_TIME_MS: u64 = 700; // ミリ秒で指定 700msに意味はないよ
    loop {
        let val = generator();
        let digi_frame = build_frame_array(val);

        // 送信
        send(SendMode::DigimaticFrame(digi_frame), &mut *ports.tx);

        // 受信
        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                if data.is_empty() {
                    continue;
                }
                // 受信記録記録用準備
                let rx_log = RxDataLog {
                    timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                    raw_len: data.len(),
                    raw_data: format!("{:?}", data),
                    error_log: None,
                };
                rx_wtr.serialize(rx_log)?;
                rx_wtr.flush()?;

                // rx文字列(フレーム)のバリデーション
                match DigimaticFrame::try_from(data.as_str()).and_then(Measurement::try_from) {
                    Ok(measurement) => {
                        let val_f64 = measurement.to_f64();
                        // データ保存用構造体準備
                        let m_log = MeasurementLog {
                            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                            val: val_f64,
                        };
                        m_wtr.serialize(m_log)?; // 測定データ記録
                        m_wtr.flush()?;
                        println!("{} {:?} : ", measurement.raw_val, measurement.unit);
                        print_tx_rx_decode_result(val, &data, val_f64);
                    }
                    Err(e) => {
                        eprintln!("データ異常（パース失敗）: {} | 原因: {}", data, e);
                    }
                }
            }
            // タイムアウト時は何もしない
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            // それ以外のエラー
            Err(e) => {
                eprintln!("受信エラー {}", e);
            }
        }
        thread::sleep(Duration::from_millis(WATI_TIME_MS));
    }
}

// 生成データ,受信文字列,復号データを出力
fn print_tx_rx_decode_result(tx_data: f64, rx_data: &str, deco_data: f64) {
    println!(
        "[Tx] {:>6.2} mm  => [Rx] {:>6} mm  [dec] {} mm",
        tx_data,
        rx_data.trim(),
        deco_data
    );
}

///
/// ライター生成
///
fn create_log_writer(path: &str) -> Result<Writer<File>, Box<dyn std::error::Error>> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;

    Ok(WriterBuilder::new().has_headers(false).from_writer(file))
}



//
// GUI+Sim用  ほとんど同じというか tx.send(vas_f64)だけなので動いたら共通化する
pub fn run_simulation_loop_with_tx(tx: std::sync::mpsc::Sender<f64>) -> Result<(), Box<dyn std::error::Error>> {
    let mut ports = port_prepare()?;
    let mut rx_receiver = CdcReceiver::new(ports.rx);
    // ログはCLI用なので省いてもOK、必要なら残す
    
    const WAIT_TIME_MS: u64 = 700;
    loop {
        let val = generator();
        let digi_frame = build_frame_array(val);
        send(SendMode::DigimaticFrame(digi_frame), &mut *ports.tx);

        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                if data.is_empty() { continue; }
                match DigimaticFrame::try_from(data.as_str()).and_then(Measurement::try_from) {
                    Ok(measurement) => {
                        let val_f64 = measurement.to_f64();
                        if tx.send(val_f64).is_err() {
                            break; // GUI終了を検知したら停止
                        }
                    }
                    Err(e) => eprintln!("パース失敗: {}", e),
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("受信エラー {}", e),
        }
        thread::sleep(Duration::from_millis(WAIT_TIME_MS));
    }
    Ok(())
}