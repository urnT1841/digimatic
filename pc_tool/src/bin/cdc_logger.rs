//!
//!  usb-cdc で受信したデータをファイルに保存
//!

use csv::{Writer, WriterBuilder};
use serialport::SerialPort;
use std::fs::{File, OpenOptions};
use std::time::Duration;

use digimatic::communicator::{
    BAUD_RATE, CdcReceiver, StopCode, open_cdc_port, wait_until_connection,
};
use digimatic::logger::RxDataLog;

///
/// usb-cdcにつながれたPicoを探して，見つかったら接続, 流れてくるデータを記録
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reason = 'outer: loop {
        // pico 接続待ち（関数内でループ見つかるかタイムアウトまで）
        let pico_port_path = match wait_until_connection() {
            Ok(path) => path,
            Err(e) => break 'outer e, // タイムアウトなら 'outer を抜ける
        };

        // open port & 書込準備
        let rx_port = open_cdc_port(&pico_port_path, BAUD_RATE)?;
        let mut rx_receiver = CdcReceiver::new(rx_port);
        let mut rx_wtr = create_log_writer("rx_debug.csv")?;

        // logging開始
        // 戻り値が HWIssue (切断) なら、pico探しに戻る
        let stop_reason = run_logging(&mut rx_receiver, &mut rx_wtr);

        if stop_reason != StopCode::HWIssue {
            break 'outer stop_reason;
        }

        println!("\nPicoとの接続が切れました。再接続を試みます...");
    };

    // 最後にレポートを表示
    println!("\n最終ステータス: {:?}", reason);
    Ok(())
}

///
/// データ保存
///
fn run_logging(
    rx_receiver: &mut CdcReceiver, // ポートを内包したレシーバーを渡す
    rx_wtr: &mut csv::Writer<std::fs::File>,
) -> StopCode {
    // Result ではなく直接 StopCode を返すと main がスッキリします
    loop {
        match rx_receiver.read_str_measurement() {
            Ok(raw) => {
                // ここで Pico からの "STOP" 文字列をチェックする
                if raw.trim() == "STOP" {
                    return StopCode::Normal; // 物理ボタンによる停止
                }

                if let Err(e) = RxDataLog::new(&raw).save_flush(rx_wtr) {
                    eprintln!("Failed to save data: {} ", e);
                }
                println!("Logged: {}", raw);
            }
            // 切断などの致命的なエラー
            Err(e) if CdcReceiver::is_fatal_error(&e) => {
                return StopCode::HWIssue; // 再接続が必要なエラー
            }
            // タイムアウトなどの一時的なエラーは無視して継続
            _ => continue,
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
