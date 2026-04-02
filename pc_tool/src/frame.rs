//!
//! frame.rs
//!
//! デジマチックのデータフォーマット用の定数や構造体
//!
//!

use std::f64::NAN;

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

// rx frame を受ける入れ物 measurement構造体前に使う
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigimaticFrame {
    pub header: [u8; 4],
    pub sign: Sign,
    pub data: [u8; 6],
    pub point_pos: PointPosition,
    pub unit: Unit,
}

impl DigimaticFrame {
    pub fn to_measurement(&self) -> Measurement {
        Measurement {
            raw_val: std::str::from_utf8(&self.data)
                .unwrap_or("0")
                .to_string(),
            sign: self.sign,
            point: self.point_pos,
            unit: self.unit,
        }
    }

}

//これは別関数として実装しなおす
impl DigimaticFrame {
    /// バイナリ列（ニブル 13 個 / 52bit）から生成
    /// `mode="LSB"` or `"MSB"` に対応
    pub fn from_bin(nibbles: &[u8], mode: &str) -> Result<Self, Error> {
        if nibbles.len() != FRAME_LENGTH {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid binary frame length: {}", nibbles.len()),
            ));
        }

        // 既存の validator ロジックを流用してチェック
        let validated = match validator_bits(nibbles, mode) {
            Some(v) => v,
            None => return Err(Error::new(ErrorKind::InvalidData, "Binary frame validation failed")),
        };

        // MSB モードで ASCII 相当に変換して、既存の convert_ 系関数を流用
        let ascii_bytes: Vec<u8> = decode_frame(&validated, "MSB")
            .unwrap()
            .as_bytes()
            .to_vec();

        Ok(Self {
            header: ascii_bytes[D1..D5].try_into().unwrap(),
            sign: convert_sign(&ascii_bytes[D5..D6])?,
            data: ascii_bytes[D6..D12].try_into().unwrap(),
            point_pos: convert_point(&ascii_bytes[D12..D13])?,
            unit: convert_unit(&ascii_bytes[D13..D13 + 1])?,
        })
    }

}


/// バイナリ検証用ラッパー（validator_bits は LSB/MSB 既存関数）
fn validator_bits(nibbles: &[u8], mode: &str) -> Option<Vec<u8>> {
    // 既存の validator をラップ
    match mode {
        "LSB" | "MSB" => crate::validator(nibbles, mode),
        _ => None,
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub raw_val: String,      // デジマチックフレームの D4-D11
    pub sign: Sign,           // 符号
    pub point: PointPosition, // 小数点位置
    pub unit: Unit,           // 測定値単位 mm ,r inch (ただmmしか使わない
}

/// Measurement構造体の値をf64に変換
/// 失敗した場合はNaNを返す
impl Measurement {
    pub fn to_f64_checked(&self) -> Result<f64, std::num::ParseFloatError> {
        let val = self.raw_val.parse::<f64>()?;

        let divisor = 10f64.powi(self.point as i32);
        let sign_dir = match self.sign {
            Sign::Plus => 1.0,
            Sign::Minus => -1.0,
        };

        Ok((val / divisor) * sign_dir)
    }

    pub fn to_f64(&self) -> f64 {
        self.to_f64_checked().unwrap_or(NAN)
    }
}