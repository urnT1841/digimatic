//!
//! ;
//!
//!
//!

use serialport::SerialPort;
use std::{thread, time::Duration};

use digimatic::port_prepare::port_prepare;
use digimatic::scanner_of_pico_connection::find_pico_port;
use digimatic::sim::frame_array_builder::build_frame_array;
use digimatic::sim::generator::generator;
use digimatic::sim::sender::{SendMode, send};
use digimatic::validater_rx_frame::parse_rx_frame;
use digimatic::{frame::*, scanner_of_pico_connection};

fn main() {
    // PC内で完結して動作確認できる
    //run_simmulation_loop();

    //実機とつないであれこれ
    run_actual_loop();
}

fn run_actual_loop() -> Result<(), Box<dyn std::error::Error>> {
    // picoが接続されているポートを走査
    // 見つからなかったらpanicで終わる。 // TODO: 後でエラーハンドリング実装すること
    let pico_port_path = find_pico_port()?;
    let mut rx_port = open_pico_port(&pico_port_path)?;
    let mut rx_receiver = CdcReceiver::new(rx_port);

    loop {
        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                if data.is_empty() {
                    continue;
                }

                // rx文字列(フレーム)のバリデーション
                match parse_rx_frame(&data) {
                    Ok(measurement) => {
                        let val_f64 = measurement.to_f64();
                        println!("{} {:?} : ", measurement.raw_val, measurement.unit);
                        print_rx_decodo_result(&data, val_f64);
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

    Ok(())
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

fn run_simmulation_loop() {
    // ポート準備
    let mut ports = port_prepare().expect("Faild to open ports");

    // 受信用構造体
    // port所有権が rx_receiverへ移る
    let mut rx_receiver = CdcReceiver::new(ports.rx);

    const WATI_TIME: u64 = 10;
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
                        println!("{} {:?} : ", measurement.raw_val, measurement.unit);
                        print_tx_rx_decodo_result(val, &data, val_f64);
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
fn print_tx_rx_decodo_result(tx_data: f64, rx_data: &str, deco_data: f64) {
    println!(
        "[Tx] {:>6.2} mm  => [Rx] {:>6} mm  [dec] {} mm",
        tx_data,
        rx_data.trim(),
        deco_data
    );
}

fn print_rx_decodo_result(rx_data: &str, deco_data: f64) {
    println!("[Rx] {:>6} mm  [dec] {} mm", rx_data.trim(), deco_data);
}
