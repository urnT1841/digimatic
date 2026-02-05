//------------------  ここからSender

//
// 仮想ポートにデータを流しこむ
//
//

use serialport::SerialPort;

/// 送信モード
pub enum SendMode {
    /// デバッグ用：単純なテキスト形式 (例: "123.45\n")
    SimpleText(f64),
    /// 本番用：デジマチック・13デジット形式 (例: "FFFF001234520\n")
    DigimaticFrame([u8; 13]),
}

/// 物理的な送信を担う内部関数（非公開）
fn write_to_port(packet: String, tx_p: &mut dyn SerialPort) {
    match tx_p.write_all(packet.as_bytes()) {
        Ok(_) => {
            if let Err(e) = tx_p.flush() {
                eprintln!("フラッシュ失敗: {}", e);
            }
        }
        Err(e) => eprintln!("送信失敗: {}", e),
    }
}

/// 外部から呼び出す窓口関数
pub fn send(mode: SendMode, tx_p: &mut dyn SerialPort) {
    let packet = match mode {
        SendMode::SimpleText(val) => format!("{:.2}\n", val),

        // デジマチック形式の構築 (13デジットをHex文字列へ)
        SendMode::DigimaticFrame(frame) => {
            let hex: String = frame.iter()
                .map(|b| format!("{:X}", b)) // 各要素を16進数文字へ
                .collect();
            format!("{}\n", hex)
        }
    };

    write_to_port(packet, tx_p);
}