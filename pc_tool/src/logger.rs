//!
//! Logやデータ保存などファイル書き込み系を扱う
//! 

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

/// 測定データ保存用
#[derive(Serialize, Debug)]
pub struct MeasurementLog {
    pub timestamp: String,
    pub val: f64,
}