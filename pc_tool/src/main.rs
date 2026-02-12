//!
//! ;
//!
//!
//!

use std::{thread, time::Duration};

use digimatic::decoder_digi_frame::decode_digi_frame_string;
use digimatic::frame_array_builder::build_frame_array;
use digimatic::generator::generator;
use digimatic::port_prepare::port_prepare;
use digimatic::receiver::receiver;
use digimatic::sender::{SendMode, send};

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

        // sender には tx port を貸し出してデータ送出  (返値なし)
        send(SendMode::DigimaticFrame(digi_frame), &mut *ports.tx); // 本番：digimaticフレームを送る
        // send(SendMode::SimpleText(val), &mut *tx_p);                            // デバッグ：生データを送る

        // reveiver には rx portを貸し出してデータ受信
        let r_data = receiver(&mut *ports.rx);
        match r_data {
            Ok(data) => {
                let decoded_result = decode_digi_frame_string(&data);

                if let Ok(mes_val) = decoded_result {
                    print_tx_rx_decodo_result(val, &data, mes_val)
                } else {
                    // エラー処理
                    eprintln!("データ異常 {}", data);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => {
                eprintln!("受信エラー {}", e)
            }
        };
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
