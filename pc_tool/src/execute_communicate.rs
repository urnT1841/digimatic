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
use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::frame::{DigimaticFrame, Measurement};
use crate::logger::*;
use crate::presentation::format_with_display_unit;
use crate::scanner::find_pico_port;

#[derive(Clone, Copy, Debug)]
pub enum FrameFormat {
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
    let _ = tx; // for gui
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

        let mut rx_receiver = CdcReceiver::new(rx_port, frame_mode);
        // 保存用にライター準備
        let mut rx_wtr = Some(create_log_writer("rx_log.csv")?);
        let mut m_wtr = Some(create_log_writer("measurement.csv")?);

        // 受信と処理
        if let Err(e) = data_receiver(frame_mode, &mut rx_receiver, &tx, &mut rx_wtr, &mut m_wtr) {
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

fn data_receiver(
    frame_mode: FrameFormat,
    rx_receiver: &mut CdcReceiver,
    tx: &std::sync::mpsc::Sender<Measurement>,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
) -> Result<(), DigimaticError> {
    loop {
        // raw data 取得に専念
        // 受信データがstr/binを問わず cdc receiverはVec<u8>を返してくる
        let raw_data = match rx_receiver.read_measurement() {
            Ok(data) => data,
            // timeoutは無視
            Err(DigimaticError::Comm(crate::errors::CommError::Timeout)) => continue,
            // 上記以外は致命扱いで上位へエラー上げる
            Err(e) => return Err(e),
        };

        // データをハンドラに投げる ここでは投げるだけで処理・解釈等は行わない
        // 生ログ保存
        if let Err(e) =
            handle_received_data(&raw_data, rx_wtr, m_wtr, &Some(tx.clone()), frame_mode)
        {
            if e.is_fatal() {
                return Err(e);
            }
            eprintln!("[Warning] data process error (Log saved) : {}", e);
        }
    }
}

/// 受信データに対する「保存・パース・送信」の共通ハンドラ
pub fn handle_received_data(
    raw_data: &[u8],
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<Measurement>>,
    format: FrameFormat,
) -> Result<(), DigimaticError> {
    // 鑑定・解析
    let (measurement_result, raw_str_for_log) = match format {
        FrameFormat::Str => {
            if !raw_data.is_ascii() {
                return Err(DigimaticError::from(FrameParseError::NonAscii));
            }
            let s = std::str::from_utf8(raw_data).map_err(|_| FrameParseError::NonAscii)?;
            let trimmed = s.trim();
            (
                DigimaticFrame::try_from(trimmed).and_then(Measurement::try_from),
                trimmed.to_string(), // String にして所有権を持つ
            )
        }
        FrameFormat::Bin => {
            let res = crate::parser::parse_bits(raw_data, crate::frame::BitMode::Lsb)
                .and_then(|nibbles| DigimaticFrame::try_from(&nibbles[..]))
                .and_then(Measurement::try_from);
            (res, hex::encode(raw_data)) // ログ用は16進数文字列
        }
    };

    // ログ保存（&str が必要なので &raw_str_for_log を渡す）
    handle_save_raw_log(&raw_str_for_log, rx_wtr, None)?;

    match measurement_result {
        Ok(m) => {
            handle_save_measurement_data(m, m_wtr, tx)?;
            println!("Decoded: {}", format_with_display_unit(&m, m.unit));
            Ok(())
        }
        Err(e) => {
            // 画像のエラー解消: &[u8] ではなく &str を渡す
            handle_save_raw_log(&raw_str_for_log, rx_wtr, Some(&e))?;
            // data 未定義エラー解消: raw_str_for_log を使う
            eprintln!("[Error] Parse Failed: {} | Raw: {}", e, raw_str_for_log);
            Err(e.into())
        }
    }
}

/// 生データ保存
fn handle_save_raw_log(
    data: &str,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    err: Option<&FrameParseError>,
) -> Result<(), DigimaticError> {
    let mut rx_log = RxDataLog::new_str(data);

    if let Some(e) = err {
        rx_log.error_log = Some(e.clone());
    }

    if let Some(w) = rx_wtr {
        rx_log.save(w)?;
    }

    Ok(())
}

/// 計測値保存 + GUIへのデータ送信 (txに流し込み))
fn handle_save_measurement_data(
    m: Measurement,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    if let Some(w) = m_wtr {
        MeasurementLog::new(m.to_f64()).save(w)?;
    }

    if let Some(t) = tx {
        t.send(m.clone()).map_err(|_| {
            DigimaticError::System(crate::errors::SystemError {
                code: 99,
                message: "Channel closed".into(),
            })
        })?;
    }

    println!("[DATA] {}", format_with_display_unit(&m, m.unit));

    Ok(())
}
