//!
//! 実際に外部機器(pico)と通信して処理する
//!

use csv::{Writer, WriterBuilder};
use serialport::SerialPort;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::time::Duration;

use crate::communicator::CdcReceiver;
use crate::logger::*;
use crate::scanner_of_pico_connection::find_pico_port;
use crate::validater_rx_frame::parse_rx_frame;

///
/// pico 実機を探して接続，USB-CDCで待ち受けデータ受信
///
pub fn run_actual_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut pico_waiting = 0;
    //pico待ち受けループ
    loop {
        // 待ち受け時間制限 10分 600s で設定
        if pico_waiting > 600 {
            println!("タイムアウト： 待ち受けを終了します。");
            break Ok(());
        }

        print!("\rpicoを探しています。{}秒 ", pico_waiting);
        io::stdout().flush().unwrap();

        // picoを探す
        let pico_port_path = match find_pico_port() {
            Ok(path) => path,
            Err(_) => {
                std::thread::sleep(Duration::from_secs(1));
                pico_waiting += 1;
                continue;
            }
        };
        // 見つかったのでリセット
        pico_waiting = 0;

        // port open
        let rx_port = match open_pico_port(&pico_port_path) {
            Ok(port) => port,
            Err(_) => continue,
        };
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

                    // 受信記録記録を生成して記録
                    //  コンストラクタと .save_flash() メソッド を impl
                    RxDataLog::new(&data).save_flush(&mut rx_wtr)?; // エラーの扱い注意

                    // rx文字列(フレーム)のバリデーション
                    match parse_rx_frame(&data) {
                        Ok(measurement) => {
                            let val_f64 = measurement.to_f64();
                            MeasurementLog::new(val_f64).save_flush(&mut m_wtr)?; // エラーの扱い注意
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
}

///
/// ライター生成
///
fn create_log_writer(path: &str) -> Result<Writer<File>, Box<dyn std::error::Error>> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;

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
