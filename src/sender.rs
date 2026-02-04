//------------------  ここからSender

//
// 仮想ポートにデータを流しこむ
//
//

use serialport::SerialPort;

pub fn send_data(data: f64, tx_p: &mut dyn SerialPort) {
    // 送信用にデータ変換
    let msg = format!("{:.2}\n", data);
    match tx_p.write_all(msg.as_bytes()) {
        Ok(_) => {
            let _ = tx_p.flush();
        }
        Err(e) => {
            eprintln!("失敗 {}", e)
        }
    }
}
