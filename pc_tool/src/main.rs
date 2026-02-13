//!
//! ;
//!
//!
//!

use std::{thread, time::Duration};

use digimatic::frame_array_builder::build_frame_array;
use digimatic::generator::generator;
use digimatic::port_prepare::port_prepare;
use digimatic::receiver::receiver;
use digimatic::sender::{SendMode, send};
use digimatic::validater_rx_frame::parse_rx_frame;

fn main() {
    // ポート準備
    let mut ports = match port_prepare() {
        Ok(port) => {
            println!("tx : {}, rx : {}", port.tx_path, port.rx_path);
            port
        }
        Err(e) => {
            eprintln!("ポートを開くのを失敗 {}", e);
            std::process::exit(1);
        }
    };

    loop {
        let val = generator();
        let digi_frame = build_frame_array(val);

        // 送信
        send(SendMode::DigimaticFrame(digi_frame), &mut *ports.tx);

        // 受信
        let r_data = receiver(&mut *ports.rx);

        match r_data {
            Ok(data) => {
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
        } // match r_data の閉じ
        thread::sleep(Duration::from_secs(1));
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
