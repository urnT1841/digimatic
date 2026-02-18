//!
//! 実際に外部機器(pico)と通信して処理する
//! 

use serialport::SerialPort;
use std::time::Duration;
use std::fs::{File,OpenOptions};
use csv::{Writer, WriterBuilder};
use chrono::Local;

use crate::scanner_of_pico_connection::find_pico_port;
use crate::validater_rx_frame::parse_rx_frame;
use crate::frame::*;


pub fn run_actual_loop() -> Result<(), Box<dyn std::error::Error>> {
    // picoが接続されているポートを走査
    // 見つからなかったらpanicで終わる。 // TODO: 後でエラーハンドリング実装すること
    let pico_port_path = find_pico_port()?;
    let rx_port = open_pico_port(&pico_port_path)?;
    let mut rx_receiver = CdcReceiver::new(rx_port);

    // 保存用にライター準備
    let mut rx_wtr = create_log_writer("rx_log.csv")?;
    let mut m_wtr = create_log_writer("measurement.csv")?;

    loop {
        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                if data.is_empty() {
                    continue;
                }

                // 受信記録記録用準備
                let rx_log = RxDataLog {
                    timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                    raw_len: data.len(),
                    raw_data: format!("{:?}",data),
                    error_log: None,
                };
                rx_wtr.serialize(rx_log)?;
                rx_wtr.flush()?;

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
                        println!("{} {:?} : ", measurement.raw_val, measurement.unit);
                        print_rx_decode_result(&data, val_f64);
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
    }
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

///
/// portのpathを受け取って Open する
///
fn open_pico_port(path: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let port = serialport::new(path, 115200)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}

// 生成データ,受信文字列,復号データを出力
fn print_rx_decode_result(rx_data: &str, deco_data: f64) {
    println!("[Rx] {:>6} mm  [dec] {} mm", rx_data.trim(), deco_data);
}
