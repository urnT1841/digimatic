//!
//! Logやデータ保存などファイル書き込み系を扱う
//! 

use serde::Serialize;
use chrono::Local;
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
    // ループ内で使用するので raw は所有権もらう move
    // 将来使いまわすとき(stdoutやGUI実装など)借用にして拡張
    pub fn new(raw: &str) -> Self {
        Self {
            timestamp: Local::now().format("%H:%M:%S.3f").to_string(),
            raw_len: raw.len(),
            raw_data: format!("{:?}", raw),
            error_log: None,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush (
        &self,
        wtr: &mut csv::Writer<std::fs::File>
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
            timestamp: Local::now().format("%H:%M:%S.3f").to_string(),
            val,
        }
    }

    // ライターにデータ送ってFlush
    pub fn save_flush (
        &self,
        wtr: &mut csv::Writer<std::fs::File>
    ) -> Result<(), Box<dyn std::error::Error>> {
        wtr.serialize(self)?;
        wtr.flush()?;
        Ok(())
    }
}