


//------------------  ここからSender

//
// 仮想ポートにデータを流しこむ
// 
// 


use std::io::Write;  // for write_all, flush
use std::thread;     // for thread::sleep
use std::time::Duration; // for duration(時間調整)
use serialport;

use crate::port_prepare::PortPair;

pub fn send_data( data:f64, port: &PortPair ) {

    // 送信用ポートをオープンする
    let mut sp = serialport::new(&port.source,9600)
                                    .timeout(Duration::from_millis(100))
                                    .open().expect("送信用ポートオープン失敗");

    // 送信用にデータ変換                                    
    let msg = format!("{:.2}\n", data);
    sp.write_all(msg.as_bytes()).unwrap();
    sp.flush().unwrap();

    println!("送信データ：{}", msg);

    // 4. 受信側が読みやすいように1秒待機
    thread::sleep(Duration::from_secs(1));

}