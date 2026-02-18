//!
//! frame.rs
//!
//! デジマチックのデータフォーマット用の定数や構造体
//!
//!

// デジマッチック データフレームの位置
// インデックスだとずれるので
pub const D1: usize = 0; // header
pub const D2: usize = 1; // header
pub const D3: usize = 2; // header
pub const D4: usize = 3; // header
pub const D5: usize = 4; // sign ( + or - )
pub const D6: usize = 5; // data
pub const D7: usize = 6; // data
pub const D8: usize = 7; // data
pub const D9: usize = 8; // data
pub const D10: usize = 9; // data
pub const D11: usize = 10; // data
pub const D12: usize = 11; // point position
pub const D13: usize = 12; // unit  ( mm or inch )

pub const FRAME_LENGTH: usize = 13; // デジマチックフレームの長さは13固定

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus = 0x00,
    Minus = 0x08,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Mm = 0x00,
    _Inch = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointPosition {
    Zero = 0x00,  // 000000.
    One = 0x01,   // 00000.0
    Two = 0x02,   // 0000.00
    Three = 0x03, // 000.000
    Four = 0x04,  // 00.0000
    Five = 0x05,  // 0.00000
}

// 作ったけど使ってない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigimaticFrame {
    pub header: [u8; 4],
    pub sign: Sign,
    pub data: [u8; 6],
    pub point_pos: PointPosition,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub raw_val: String,      // デジマチックフレームの D4-D11
    pub sign: Sign,           // 符号
    pub point: PointPosition, // 小数点位置
    pub unit: Unit,           // 測定値単位 mm ,r inch (ただmmしか使わない
}

impl Measurement {
    pub fn to_f64(&self) -> f64 {
        // 整数として保存している部分を数値に変換
        let val = self.raw_val.parse::<f64>().unwrap_or(0.0);

        //小数点の桁数分で割って測定値に変換。そのあと符号適用
        let divisor = 10f64.powi(self.point as i32);
        let sign_dir = match self.sign {
            Sign::Plus => 1.0,
            Sign::Minus => -1.0,
        };

        // 最終的な f64 の測定値
        (val / divisor) * sign_dir
    }
}

use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::time::Duration;

#[derive(Debug)]
pub struct CdcReceiver {
    rx_reader: BufReader<Box<dyn SerialPort>>,
}

impl CdcReceiver {
    pub fn new(mut port: Box<dyn SerialPort>) -> Self {
        port.set_timeout(Duration::from_millis(2000))
            .expect("Failed to set timeout");
        Self {
            rx_reader: BufReader::new(port),
        }
    }

    /// 文字列で送信されたデータを1行読み込む
    pub fn read_str_measurement(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        // read_lineは改行が来るまで待機
        match self.rx_reader.read_line(&mut line) {
            Ok(0) => Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Pico disconnected",
            )),
            Ok(_) => Ok(line.trim().to_string()),
            Err(e) => Err(e),
        }
    }

    // バイナリで送信されたデータ受信 \n終端前提
    pub fn read_bin_measurment(&mut self) -> Result<Vec<u8>, Error> {
        let mut rx_stream = Vec::new();

        match self.rx_reader.read_until(b'\n', &mut rx_stream) {
            Ok(0) => Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Pico disconnected",
            )),
            Ok(n) => {
                if n > 32 {
                    return Err(Error::new(ErrorKind::InvalidData, "Frame too long"));
                }
                // trim_ascii_end を使って末尾を綺麗にする
                Ok(rx_stream.trim_ascii_end().to_vec())
            }
            Err(e) => Err(e),
        }
    }
}


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