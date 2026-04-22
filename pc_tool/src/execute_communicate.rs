//!
//! 実際に外部機器(pico)と通信して処理する
//!

use csv::{Writer, WriterBuilder};
use serialport::SerialPort;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::communicator::CdcReceiver;
use crate::communicator::MeasurementRead;
use crate::errors::{CommError, DigimaticError};
use crate::frame::{DigimaticFrame, Measurement};
use crate::logger::*;
use crate::parser;
use crate::scanner::find_pico_port;

#[derive(Clone, Copy)]
enum FrameFormat {
    Str,
    Bin,
}

///
/// pico 実機を探して接続，USB-CDCで待ち受けデータ受信
///
pub fn run_actual_loop(
    tx: std::sync::mpsc::Sender<Measurement>, // guiへデータ送るため
) -> Result<(), DigimaticError> {
    let frame_mode: FrameFormat = FrameFormat::Bin;
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
        println!("\rPicoを発見しました! 接続します。... ");

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
        if let Err(e) = receiver(frame_mode, &mut rx_receiver, &tx, &mut rx_wtr, &mut m_wtr) {
            if e.is_fatal() {
                return Err(e); // エラーで致命なら終了
            }
            // 致命エラーが出なければ続ける (pico捜索から)
            if let DigimaticError::Comm(crate::errors::CommError::Timeout) = e {
                // ここは何もしない
            } else {
                eprintln!("エラーが出ましたが，そのまま続行します: {}", e);
            }
            continue;
        }
    }
}

///
/// ライター生成
///
pub fn create_log_writer(path: &str) -> Result<Writer<File>, CommError> {
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

/// data receiver & dispath
fn receiver(
    frame_mode: FrameFormat,
    rx_receiver: &mut CdcReceiver,
    tx: &std::sync::mpsc::Sender<Measurement>,
    rx_wtr: &mut csv::Writer<std::fs::File>,
    m_wtr: &mut csv::Writer<std::fs::File>,
) -> Result<(), DigimaticError> {
    match frame_mode {
        FrameFormat::Str => process_string_frame(rx_receiver, tx, rx_wtr, m_wtr),
        FrameFormat::Bin => process_binary_frame(rx_receiver, tx, rx_wtr, m_wtr),
    }
}
//
/// data pcocesser
///
fn process_string_frame(
    rx_receiver: &mut CdcReceiver,
    tx: &std::sync::mpsc::Sender<Measurement>,
    rx_wtr: &mut csv::Writer<std::fs::File>,
    _m_wtr: &mut csv::Writer<std::fs::File>,
) -> Result<(), DigimaticError> {
    loop {
        match rx_receiver.read_str_measurement() {
            Ok(data) => {
                // 受信記録を生成してCSVに保存
                if let Err(e) = RxDataLog::new_str(&data).save_flush(rx_wtr) {
                    eprintln!("Failed to save raw data: {}", e);
                }

                // 文字列フレームをDigimaticFrameに変換 -> Measurement構造体へ
                match DigimaticFrame::try_from(data.as_str()) {
                    Ok(frame) => {
                        match Measurement::try_from(frame) {
                            Ok(measurement) => {
                                // ターミナルへの表示とGUIへ送信
                                println!(
                                    "[Rx] {} -> [dec] {:.2} mm",
                                    data.trim(),
                                    measurement.to_f64()
                                );
                                let _ = tx.send(measurement);
                            }
                            Err(e) => eprintln!("Measurement 変換エラー: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Frame パースエラー: {} | 原因: {}", data, e),
                }
            }

            // タイムアウトは無視
            Err(DigimaticError::Comm(CommError::Timeout)) => {
                //何もしない
            }
            Err(e) => {
                if e.is_fatal() {
                    // 致命的エラーなら上に持ち上げる
                    eprintln!(
                        "[Waring] エラーが発生しましたが，続行します。 エラー：{}",
                        e
                    );
                    return Err(e);
                }
            }
        }
    }
}

fn process_binary_frame(
    rx_receiver: &mut CdcReceiver,
    tx: &std::sync::mpsc::Sender<Measurement>,
    rx_wtr: &mut csv::Writer<std::fs::File>,
    _m_wtr: &mut csv::Writer<std::fs::File>,
) -> Result<(), DigimaticError> {
    // とりあえずコンパイル通すために入れる。
    // あとで取り除く
    let bit_mode = crate::frame::BitMode::Lsb;

    loop {
        match rx_receiver.read_bin_measurement() {
            Ok(data) => {
                // 保存用ログ（実装済みの new_bin を使用）
                if let Err(e) = RxDataLog::new_bin(&data).save_flush(rx_wtr) {
                    eprintln!("Failed to save raw data: {}", e);
                }

                // 52bit -> 13ニブル
                // parse_bits は Result を返すので ? で抜けるか match で受ける
                match parser::parse_bits(&data, bit_mode) {
                    Ok(nibbles) => {
                        //  検証 & 構造体化: &nibbles[..] でスライスとして渡す
                        match DigimaticFrame::try_from(&nibbles[..]) {
                            Ok(frame) => {
                                if let Ok(measurement) = Measurement::try_from(frame) {
                                    // バイナリなので data.trim() ではなく 変換後の nibbles を表示
                                    println!(
                                        "[Bin Rx] {:?} -> {:.2} mm",
                                        nibbles,
                                        measurement.to_f64()
                                    );
                                    let _ = tx.send(measurement);
                                }
                            }
                            Err(e) => eprintln!("Frame パースエラー(Bin): {}", e),
                        }
                    }
                    Err(e) => eprintln!("ニブル変換エラー: {}", e),
                }
            }
            Err(DigimaticError::Comm(CommError::Timeout)) => {
                // 何もしない
            }
            Err(e) => {
                if e.is_fatal() {
                    eprintln!(
                        "[Waring] エラーが発生しましたが，続行します。 エラー：{}",
                        e
                    );
                    return Err(e);
                }
                continue;
            }
        }
    }
}

/// 受信データに対する「保存・パース・送信」の共通ハンドラ
/// execute_sim から移設  → データハンドリングの要として昇格
pub fn handle_received_data(
    data: &str,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    // 生ログの準備 (時刻はこの瞬間に固定)
    let mut rx_log = RxDataLog::new_str(data);

    // 計測データ → フレーム生成 (TryFrom chain)
    match DigimaticFrame::try_from(data).and_then(Measurement::try_from) {
        Ok(m) => {
            //  生データの保存 (Writerがあれば)
            if let Some(w) = rx_wtr {
                rx_log.save_flush(w)?;
            }

            // 測定値の保存 (Writerがあれば)
            if let Some(w) = m_wtr {
                MeasurementLog::new(m.to_f64()).save_flush(w)?;
            }

            // GUIへの送信 (Senderがあれば)
            if let Some(t) = tx {
                t.send(m.clone()).map_err(|_| {
                    DigimaticError::System(crate::errors::SystemError {
                        code: 99,
                        message: "Channel closed".into(),
                    })
                })?;
            }
            // cli表示(共通化で邪魔になったら調整)
            println!("[SIM] Decoded: {:.3} mm", m.to_f64());
        }
        Err(e) => {
            // パース失敗時：エラーを載せて生ログだけは残す
            if let Some(w) = rx_wtr {
                rx_log.error_log = Some(e.clone());
                rx_log.save_flush(w)?;
            }
            eprintln!("[SIM] Parse Error: {} | Raw: {}", e, data);
            return Err(DigimaticError::from(e));
        }
    }
    Ok(())
}
