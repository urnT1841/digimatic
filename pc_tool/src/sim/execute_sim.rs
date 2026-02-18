//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの 
//! 

use std::{thread, time::Duration};
use std::fs::{File,OpenOptions};
use csv::{Writer, WriterBuilder};
use chrono::Local;

use crate::sim::frame_array_builder::build_frame_array;
use crate::sim::generator::generator;
use crate::sim::port_prepare::port_prepare;
use crate::sim::sender::{SendMode, send};
use crate::validater_rx_frame::parse_rx_frame;
use crate::frame::*;


pub fn run_simmulation_loop() -> Result<(),Box<dyn std::error::Error>> {
    // ポート準備
    let mut ports = port_prepare().expect("Faild to open ports");

    // 受信用構造体
    // port所有権が rx_receiverへ移る
    let mut rx_receiver = CdcReceiver::new(ports.rx);

    // 保存用にライター準備
    let mut rx_wtr = create_log_writer("rx_log.csv")?;
    let mut m_wtr = create_log_writer("measurement.csv")?;


    const WATI_TIME: u64 = 1;  // 秒で指定
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

                // rx文字列(フレーム)のバリデーション
                match parse_rx_frame(&data) {
                    Ok(measurement) => {
                        let val_f64 = measurement.to_f64();
                        // データ保存用構造体準備
                        let m_log = MeasurementLog {
                            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                            val: val_f64,
                        };
                        m_wtr.serialize(m_log)?;  // 測定データ記録
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
        thread::sleep(Duration::from_secs(WATI_TIME));
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
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    Ok(WriterBuilder::new().has_headers(false).from_writer(file))
}