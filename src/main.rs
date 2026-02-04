//!
//! ;
//!
//!
//!

mod port_prepare;
use std::{thread, time::Duration};

use port_prepare::port_prepare;

mod generator;
use generator::generator;
mod sender;
use sender::send_data;

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

        // sender には tx port を貸し出す
        send_data(val, &mut *ports.tx);
        thread::sleep(Duration::from_secs(1));
    }

    // println!("✅ 送信完了。Enterキーを押すと終了します（ポートを閉じます）...");
    // let mut buffer = String::new();
    // std::io::stdin().read_line(&mut buffer).unwrap();
}
