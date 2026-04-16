//!
//! Logやデータ保存などファイル書き込み系を扱う
//!

use chrono::Local;
use serde::Serialize;

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
    // コンストラクタ
    pub fn new(raw: &str) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339().to_string(),
            raw_len: raw.len(),
            raw_data: raw.to_string(),
            error_log: None,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush(
        &self,
        wtr: &mut csv::Writer<std::fs::File>,
    ) -> Result<(), DigimaticError> {
        //csv::Errorを systemErrorでラップする
        wtr.serialize(self).map_err(|e| {
            SystemError {
                code: 101,
                message: format!("CSV serializatino failed: {}", e),
            }
        })?;
        
        wtr.flush().map_err(|e| {
            SystemError {
                code: 102,
                message: format!("CSV flash failed: {}",e),
            }
        })?;
        Ok(())
    }
}

/// 測定データ保存用
#[derive(Serialize, Debug, Clone)]
pub struct MeasurementLog {
    pub timestamp: String,
    pub val: f64,
}

impl MeasurementLog {
    // コンストラクタ
    pub fn new(val: f64) -> Self {
        Self {
            timestamp: Local::now().to_rfc3339().to_string(),
            val,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush(
        &self,
        wtr: &mut csv::Writer<std::fs::File>,
    ) -> Result<(), DigimaticError> {
        //csv::Errorを systemErrorでラップする
        wtr.serialize(self).map_err(|e| {
            SystemError {
                code: 101,
                message: format!("CSV serializatino failed: {}", e),
            }
        })?;
        
        wtr.flush().map_err(|e| {
            SystemError {
                code: 102,
                message: format!("CSV flash failed: {}",e),
            }
        })?;
        Ok(())
    }
}
