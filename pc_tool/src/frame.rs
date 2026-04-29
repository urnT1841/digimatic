//!
//! frame.rs
//!
//! デジマチックのデータフォーマット用の定数や構造体
//!
//!

use std::f64::NAN;

use crate::errors::FrameParseError;

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
pub const FRAME_NIBBLES: usize = 4; // デジマチックフレームの1つは4Bit (nibble)

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus = 0x00,
    Minus = 0x08,
}

impl TryFrom<u8> for Sign {
    type Error = FrameParseError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(Sign::Plus),
            0x08 => Ok(Sign::Minus),
            _ => Err(FrameParseError::InvalidSign),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Mm = 0x00,
    Inch = 0x01,
}

impl TryFrom<u8> for Unit {
    type Error = FrameParseError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(Unit::Mm),
            0x01 => Ok(Unit::Inch),
            _ => Err(FrameParseError::InvalidUnit),
        }
    }
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

impl TryFrom<u8> for PointPosition {
    type Error = FrameParseError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(Self::Zero),
            0x01 => Ok(Self::One),
            0x02 => Ok(Self::Two),
            0x03 => Ok(Self::Three),
            0x04 => Ok(Self::Four),
            0x05 => Ok(Self::Five),
            _ => Err(FrameParseError::InvalidPoint),
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub raw_val: String,      // デジマチックフレームの D4-D11
    pub sign: Sign,           // 符号
    pub point: PointPosition, // 小数点位置
    pub unit: Unit,           // 測定値単位 mm ,r inch (ただmmしか使わない
}

// 初期化 (コンストラクタ)
impl Measurement {
    pub fn dummy() -> Self {
        Self {
            raw_val: "999999".to_string(),
            sign: Sign::Plus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        }
    }
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
        // 元は異常値の時はNaNを返していたが,Valideto_bcdを導入したので
        // そもそも異常値は来なくなった。 → ここは失敗しない
        self.to_f64_checked().unwrap()
    }
}

/// ビット並び順モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitMode {
    Lsb,
    Msb,
}
