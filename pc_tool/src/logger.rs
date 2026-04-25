//!
//! Logやデータ保存などファイル書き込み系を扱う
//!

use chrono::Local;
use serde::Serialize;
use std::fs::File;

use crate::errors::{DigimaticError, FrameParseError, SystemError};

/// 通信データ保存用
#[derive(Serialize, Debug)]
pub struct RxDataLog {
    pub timestamp: String,
    pub raw_len: usize,
    pub raw_data: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_log: Option<FrameParseError>,
}

impl RxDataLog {
    /// 文字列データのコンストラクタ
    pub fn new_str(raw: &str) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339(),
            raw_len: raw.len(),
            raw_data: raw.to_string(),
            error_log: None,
        }
    }

    /// バイナリデータのコンストラクタ
    pub fn new_bin(raw: &[u8]) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339(),
            raw_len: raw.len(),
            raw_data: hex::encode(raw),
            error_log: None,
        }
    }

    /// CSV保存（即flush保証）
    pub fn save(&self, wtr: &mut csv::Writer<File>) -> Result<(), DigimaticError> {
        write_csv_and_flush(wtr, self)
    }
}


/// 測定データ保存用
#[derive(Serialize, Debug, Clone)]
pub struct MeasurementLog {
    pub timestamp: String,
    pub val: f64,
}

impl MeasurementLog {
    /// コンストラクタ
    pub fn new(val: f64) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339(),
            val,
        }
    }

    /// CSV保存（即flush保証）
    pub fn save(&self, wtr: &mut csv::Writer<File>) -> Result<(), DigimaticError> {
        write_csv_and_flush(wtr, self)
    }
}

// IO(CSV書き込み+Flush)を共通化
fn write_csv_and_flush<T: Serialize>(
    wtr: &mut csv::Writer<File>,
    value: &T,
) -> Result<(), DigimaticError> {
    wtr.serialize(value).map_err(|e| SystemError {
        code: 101,
        message: format!("CSV serialization failed: {}", e),
    })?;

    wtr.flush().map_err(|e| SystemError {
        code: 102,
        message: format!("CSV flush failed: {}", e),
    })?;

    Ok(())
}
