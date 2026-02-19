//!
//! Logやデータ保存などファイル書き込み系を扱う
//!

use chrono::Local;
use serde::Serialize;
//use std::io::{BufRead, BufReader, Error, ErrorKind};

/// 通信データ保存用
#[derive(Serialize, Debug)]
pub struct RxDataLog {
    pub timestamp: String,
    pub raw_len: usize,
    pub raw_data: String,
    pub error_log: Option<String>,
}

impl RxDataLog {
    // コンストラクタ
    pub fn new(raw: &str) -> Self {
        Self {
            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
            raw_len: raw.len(),
            raw_data: format!("{:?}", raw),
            error_log: None,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush(
        &self,
        wtr: &mut csv::Writer<std::fs::File>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        wtr.serialize(self)?;
        wtr.flush()?;
        Ok(())
    }
}

/// 測定データ保存用
#[derive(Serialize, Debug)]
pub struct MeasurementLog {
    pub timestamp: String,
    pub val: f64,
}

impl MeasurementLog {
    // コンストラクタ
    pub fn new(val: f64) -> Self {
        Self {
            timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
            val,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush(
        &self,
        wtr: &mut csv::Writer<std::fs::File>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        wtr.serialize(self)?;
        wtr.flush()?;
        Ok(())
    }
}
