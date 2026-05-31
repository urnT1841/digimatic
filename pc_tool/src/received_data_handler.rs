//!
//! 実際に外部機器(pico)と通信して処理する
//!

use csv::{Writer, WriterBuilder};
use std::fs::{File, OpenOptions};
use std::sync::mpsc::Sender;

use crate::config::ConsoleMode;
use crate::config::FrameFormat;
use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::frame::{DigimaticFrame, Measurement, TransportFrame};
use crate::logger::*;
use crate::presentation::format_with_display_unit;

///
/// ライター生成
///
pub fn create_log_writer(path: &str) -> Result<Writer<File>, CommError> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(WriterBuilder::new().has_headers(false).from_writer(file))
}

/// 受信データに対する「保存・パース・送信」の共通ハンドラ
pub fn handle_received_data(
    receive_frame: &TransportFrame,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
    tx: &Option<Sender<Measurement>>,
    console_mode: ConsoleMode,
) -> Result<(), DigimaticError> {
    //受信した生データを検証・解析
    let (measurement_result, raw_str_for_log) = decode_raw_data(receive_frame)?;

    // 生ログ保存
    handle_save_raw_log(receive_frame, rx_wtr, None)?;

    // 計測データとGUIへのデータ送信
    match measurement_result {
        Ok(m) => {
            save_measurement_to_csv(&m, m_wtr)?;
            push_measurement_to_gui(&m, tx)?;

            crate::logger::console_info(
                console_mode,
                format!("[Decoded]: {}", format_with_display_unit(&m, m.unit)),
            );
            Ok(())
        }
        Err(e) => {
            handle_save_raw_log(receive_frame, rx_wtr, Some(&e))?;
            crate::logger::console_error(format!(
                "[Error] Parse Failed: {} | Raw: {}",
                e, raw_str_for_log
            ));
            Err(e.into())
        }
    }
}

//生データ解釈
fn decode_raw_data(
    raw_frame: &TransportFrame,
) -> Result<(Result<Measurement, FrameParseError>, String), DigimaticError> {
    let (raw_data, format) = match raw_frame {
        TransportFrame::Str(v) => (v.as_slice(), FrameFormat::Str),
        TransportFrame::Bin(v) => (v.as_slice(), FrameFormat::Bin),
    };

    let pair = match format {
        FrameFormat::Str => {
            // バリデーション
            if !raw_data.is_ascii() {
                return Err(DigimaticError::from(FrameParseError::NonAscii));
            }
            let s = std::str::from_utf8(raw_data).map_err(|_| FrameParseError::NonAscii)?;
            let trimmed = s.trim();

            // 文字列解析
            (
                DigimaticFrame::try_from(trimmed).and_then(Measurement::try_from),
                trimmed.to_string(),
            )
        }
        FrameFormat::Bin => {
            // バイナリ版の解析
            let res = crate::parser::parse_bits(raw_data, crate::frame::BitMode::Lsb)
                .and_then(|nibbles| DigimaticFrame::try_from(&nibbles[..]))
                .and_then(Measurement::try_from);
            (res, hex::encode(raw_data))
        }
    };

    Ok(pair)
}

/// 生データ保存
fn handle_save_raw_log(
    raw_frame: &TransportFrame,
    rx_wtr: &mut Option<csv::Writer<std::fs::File>>,
    err: Option<&FrameParseError>,
) -> Result<(), DigimaticError> {
    // 受信した str / bin により適切な構造体を生成
    let mut rx_log = match raw_frame {
        TransportFrame::Str(bytes) => {
            let s = std::str::from_utf8(bytes).unwrap_or("");
            RxDataLog::new_str(s.trim())
        }
        TransportFrame::Bin(bytes) => RxDataLog::new_bin(bytes),
    };

    if let Some(e) = err {
        rx_log.error_log = Some(e.clone());
    }

    if let Some(w) = rx_wtr {
        rx_log.save(w)?;
    }

    Ok(())
}

// 計測値保存
fn save_measurement_to_csv(
    m: &Measurement,
    m_wtr: &mut Option<csv::Writer<std::fs::File>>,
) -> Result<(), DigimaticError> {
    if let Some(w) = m_wtr {
        MeasurementLog::new(m.to_f64()).save(w)?;
    }
    Ok(())
}

// GUIへのデータ送信
fn push_measurement_to_gui(
    m: &Measurement,
    tx: &Option<Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    if let Some(t) = tx {
        t.send(*m).map_err(|_| {
            DigimaticError::System(crate::errors::SystemError {
                code: 99,
                message: "Channel closed".into(),
            })
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_handle_save_raw_log_binary_hex_roundtrip() {
        // test data
        // [u8] を TransportFrameでつつむ
        let dummy_raw_bytes = vec![0x00, 0x11, 0x22, 0x33, 0xAA, 0xBB, 0xCC];
        let dummy_frame = TransportFrame::Bin(dummy_raw_bytes);
        // プロジェクトルート直下にテスト用ファイルを指定
        let test_file_path = "test_rx_binary_raw.csv";

        // 前回の残骸があれば削除
        if std::path::Path::new(test_file_path).exists() {
            let _ = fs::remove_file(test_file_path);
        }

        {
            // 本番と全く同じ `Writer<File>` をルート直下に生成！
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(test_file_path)
                .unwrap();
            let wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(file);

            // 生の[u8] じゃなくて TransportFrameで包んで渡す。

            // 今回修正した関数を呼び出す（Windowsでも100%確実に通ります）

            handle_save_raw_log(&dummy_frame, &mut Some(wtr), None).unwrap();
        } // ここでファイルが完全にクローズ

        // 3. 書き出されたファイルを読み込んで検証
        let csv_string = fs::read_to_string(test_file_path).unwrap();

        // 後始末（テスト用ファイルを綺麗に削除）
        let _ = fs::remove_file(test_file_path);

        // 🌟 検証：生バイナリがちゃんと「16進数文字列」になってCSVに刻まれているか！
        let expected_hex = match &dummy_frame {
            TransportFrame::Str(v) => hex::encode(v),
            TransportFrame::Bin(v) => hex::encode(v),
        };

        assert!(
            csv_string.contains(&expected_hex),
            "CSVに出力されたログに、正しい16進数文字列が含まれていません！ 出力: {}",
            csv_string
        );
    }
}
