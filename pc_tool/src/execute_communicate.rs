//!
//! 実際に外部機器(pico)と通信して処理する
//!

use csv::{Writer, WriterBuilder};
use std::fs::{File, OpenOptions};
use std::sync::mpsc::Sender;

use crate::config::ConsoleMode;
use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::frame::{DigimaticFrame, Measurement};
use crate::logger::*;
use crate::presentation::format_with_display_unit;

#[derive(Clone, Copy, Debug)]
pub enum FrameFormat {
    Str,
    Bin,
}

///
/// ライター生成
///
pub fn create_log_writer(path: &str) -> Result<Writer<File>, CommError> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(WriterBuilder::new().has_headers(false).from_writer(file))
}

/// 受信データに対する「保存・パース・送信」の共通ハンドラ
pub fn handle_received_data(
    raw_data: &[u8],
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<Measurement>>,
    format: FrameFormat,
    console_mode: ConsoleMode,
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
            crate::logger::console_info(
                console_mode,
                format!("[Decoded]: {}", format_with_display_unit(&m, m.unit)),
            );
            Ok(())
        }
        Err(e) => {
            // 画像のエラー解消: &[u8] ではなく &str を渡す
            handle_save_raw_log(&raw_str_for_log, rx_wtr, Some(&e))?;
            // data 未定義エラー解消: raw_str_for_log を使う
            crate::logger::console_error(format!(
                "[Error] Parse Failed: {} | Raw: {}",
                e, raw_str_for_log
            ));
            //eprintln!("[Error] Parse Failed: {} | Raw: {}", e, raw_str_for_log);
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

    Ok(())
}
