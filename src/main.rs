//!
//! ;
//!
//!
//!

use std::{thread, time::Duration};

use digimatic::port_prepare::port_prepare;
use digimatic::generator::generator;
use digimatic::sender::send_data;
use digimatic::receiver::receiver;
//use serialport::SerialPort;

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
        // testのためループでじゃんじゃん出す
        let val = generator();

        // sender には tx port を貸し出してデータ送出  (返値なし)
        send_data(val, &mut *ports.tx);

        // reveiver には rx portを貸し出してデータ受信
        let r_data = match receiver(&mut *ports.rx) {
            Ok(data) => {
                println!("受信データ： {}", data.trim()); 
                data
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                continue; // タイムアウトならここでループの先頭に戻る（match外の println は実行しない）
            }
            Err(e) => {
                eprintln!("受信エラー {}", e);
                continue; // エラー時も同様
            }
        };
        // r_data は String 型になっているので、{} で表示可能
        println!("Excel/IgorProに送るデータ: {} ", r_data.trim());
        thread::sleep(Duration::from_secs(1));
    }
}
