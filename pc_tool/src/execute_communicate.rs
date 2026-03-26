//!
//! 実際に外部機器(pico)と通信して処理する
//!

use csv::{Writer, WriterBuilder};
use serialport::SerialPort;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::time::Duration;

use crate::communicator::CdcReceiver;
use crate::logger::*;
use crate::scanner::find_pico_port;
use crate::validater::parse_rx_frame;

///
/// pico 実機を探して接続，USB-CDCで待ち受けデータ受信
///
pub fn run_actual_loop(
    tx: std::sync::mpsc::Sender<f64>, // guiへデータ送るため
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pico_waiting = 0;
    //pico待ち受けループ
    loop {
        // 待ち受け時間制限 10分 600s で設定
        if pico_waiting > 600 {
            println!("タイムアウト： 待ち受けを終了します。");
            break Ok(());
        }

        print!("\rpicoを探しています。{}秒 ", pico_waiting);
        io::stdout().flush().unwrap();

        // picoを探す
        let pico_port_path = match find_pico_port() {
            Ok(path) => path,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(400));
                pico_waiting += 1;
                continue;
            }
        };
        // 見つかったのでリセット
        pico_waiting = 0;
        println!("\rPicoを発見しました! 接続sします。... ");

        // port open
        let rx_port = match open_pico_port(&pico_port_path) {
            Ok(port) => {
                println!("ポートオープン成功: {}", pico_port_path);
                port
            }
            Err(e) => {
                println!("port open fail! retry : {}", e);
                std::thread::sleep(Duration::from_millis(500)); // すぐに戻ると見失うこともあるのでちょい待ちを入れる
                continue;
            }
        };

        let mut rx_receiver = CdcReceiver::new(rx_port);
        // 保存用にライター準備
        let mut rx_wtr = create_log_writer("rx_log.csv")?;
        let mut m_wtr = create_log_writer("measurement.csv")?;

        // 受信と処理
        if let Err(e) = receiveer(&mut rx_receiver, &tx, &mut rx_wtr, &mut m_wtr) {
            if CdcReceiver::is_fatal_error(&e) {
                break Ok(()); // エラーで致命なら終了
            }
            // 地名出なければ続ける (pico捜索から)
            continue;
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

///
/// portのpathを受け取って Open する
///
fn open_pico_port(path: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let port = serialport::new(path, 115200)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}

///
/// data recievr & pcocesser
///
fn receiveer(
    rx_receiver: &mut CdcReceiver,
    tx: &std::sync::mpsc::Sender<f64>,
    rx_wtr: &mut Writer<File>,
    m_wtr: &mut Writer<File>,
) -> Result<(), std::io::Error> {
    loop {
        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                // 受信記録記録を生成して記録
                if let Err(e) = RxDataLog::new(&data).save_flush(rx_wtr) {
                    eprintln!("Failed to save data: {}", e)
                }
                // rx文字列(フレーム)のバリデーション
                match parse_rx_frame(&data) {
                    Ok(measurement) => {
                        let val_f64 = measurement.to_f64();
                        //保存失敗は表示
                        if let Err(e) = MeasurementLog::new(val_f64).save_flush(m_wtr) {
                            eprintln!("CSV保存失敗 ： {}", e);
                        }
                        // ターミナルへ表示
                        print_rx_decode_result(&data, val_f64);
                        // gui アダプタ へデータ送る
                        // 送信失敗は無視
                        let _ = tx.send(val_f64);
                    }
                    Err(e) => {
                        eprintln!("データ異常（パース失敗）: {} | 原因: {}", data, e);
                    }
                }
            }
            // タイムアウト時は何もしない
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            // それ以外の命的な場合，このループを脱げて外側に出る
            Err(e) => {
                //eprintln!("受信エラー種別: {:?} / {}", e.kind(), e);
                if CdcReceiver::is_fatal_error(&e) {
                    return Err(e);
                }
                continue;
            }
        }
    }
}

// 生成データ,受信文字列,復号データを出力
fn print_rx_decode_result(rx_data: &str, deco_data: f64) {
    println!("[Rx] {:>6} mm  [dec] {} mm", rx_data.trim(), deco_data);
}
