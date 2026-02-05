//!
//! ;
//!
//!
//!

use std::{thread, time::Duration};

use digimatic::frame_array_builder::build_frame_array;
use digimatic::port_prepare::port_prepare;
use digimatic::generator::generator;
use digimatic::sender::{send,SendMode};
use digimatic::receiver::receiver;

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
        let digi_frame =  build_frame_array(val);

        // sender には tx port を貸し出してデータ送出  (返値なし)
        // 本番：digimaticフレームを送る
        send(SendMode::DigimaticFrame(digi_frame), &mut *ports.tx);

        // デバッグ：生データを送って確認する
        // send(SendMode::SimpleText(val), &mut *tx_p);

        // reveiver には rx portを貸し出してデータ受信
        let r_data =receiver(&mut *ports.rx);
        match r_data {
            Ok(data) => {
                print_mes_result(val,&data)
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                ()
            },
            Err(e) => {
                eprintln!("受信エラー {}", e)
            },
        };
        thread::sleep(Duration::from_secs(1));
    }
}

// 生成データと受信データ出力
fn print_mes_result(tx_data: f64, rx_data: &str) {

    println!("[Tx] {:>6.2} mm  => [Rx] {:>6} mm", tx_data, rx_data.trim());

}
