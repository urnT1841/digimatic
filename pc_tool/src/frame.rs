//! # デジマチックの  データフォーマット用の定数や構造体
//! frame.rs
//!
//! # Measurement Frame Representation
//!
//! This module defines the core domain models and data structures that represent
//! parsed measurement data within the application. It acts as a bridge between
//! raw serial inputs and the presentation layer, holding typed values, units,
//! and validation metadata to ensure data integrity across the entire pipeline.
//!

use crate::errors::FrameParseError;

// デジマチック データフレームの位置
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
pub const BITS_PER_NIBBLE: usize = 4; // デジマチックフレームの1つは4Bit (nibble)

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    #[default]
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

/// 測定値のドメイン型。
///
/// フィールドはすべて非公開。
/// 「Measurementは(パース処理を経て)生成されるものであり、
///  外部から自由に組み立てられるべきものではない」という方針により、
/// 構築経路を `TryFrom<DigimaticFrame>` と `dummy()` に限定している。
/// 値を読みたいだけの場合は各getter（`val()`/`sign()`/`point()`/`unit()`）
/// または `to_f64()` を使うこと。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measurement {
    val: u32,             // デジマチックフレームの D6-D11
    sign: Sign,           // 符号
    point: PointPosition, // 小数点位置
    unit: Unit,           // 測定値単位 mm ,inch (ただmmしか使わない)
}

// 初期化 (コンストラクタ)
impl Measurement {
    //初期化用のダミー数値
    pub const DUMMY_VALUE: u32 = 999_999;

    pub fn dummy() -> Self {
        Self {
            val: Self::DUMMY_VALUE,
            sign: Sign::Plus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        }
    }
}

// getter群。
// フィールドを非公開にした代わりに、読み取り専用のアクセサを用意する。
// Measurement自体はCopyな小さい型なので、参照ではなく値で返して問題ない。
impl Measurement {
    pub fn val(&self) -> u32 {
        self.val
    }

    pub fn sign(&self) -> Sign {
        self.sign
    }

    pub fn point(&self) -> PointPosition {
        self.point
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }
}

impl Measurement {
    pub fn to_f64(self) -> f64 {
        let divisor = 10f64.powi(self.point as i32);
        let sign_dir = match self.sign {
            Sign::Plus => 1.0,
            Sign::Minus => -1.0,
        };

        self.val as f64 / divisor * sign_dir
    }
}

// Digimatic -> measurement
//
// parser.rs から移設。フィールドが非公開になったため、構造体リテラルで
// Measurementを組み立てられるのはこのモジュール(frame.rs)内だけになった。
// 「パース経由でしか生成できない」という制約を型で表現するには、この
// TryFrom実装がMeasurementと同じモジュールにある必要がある。
impl TryFrom<DigimaticFrame> for Measurement {
    type Error = FrameParseError;

    fn try_from(frame: DigimaticFrame) -> Result<Self, Self::Error> {
        // 計測値データニブルをu32に変換
        // validate_bcd_slice()で BCD数値であることが検証ずみなので失敗は想定せず
        // foldで積み重ねる
        let val = frame
            .data
            .iter()
            .fold(0u32, |acc, &nibble| acc * 10 + (nibble as u32));

        Ok(Measurement {
            val,
            sign: frame.sign,
            point: frame.point_pos,
            unit: frame.unit,
        })
    }
}

/// Bit ordering mode.
///
/// Note:
/// MSB is defined for model completeness,
/// but Digimatic protocol uses LSB-only ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitMode {
    Lsb,
    #[allow(dead_code)]
    Msb,
}

/// 受信フレームを型に押込める
#[derive(Debug, Clone)]
pub enum TransportFrame {
    Str(Vec<u8>),
    Bin(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::*;

    // parser.rs から移設したテスト群。
    // 「Measurementの構造体リテラルを直接書く」という書き方自体が
    // 非公開フィールド化によりこのモジュール内でしかできなくなったため、
    // Measurement自身の定義と同じ場所（frame.rs）に置くのが自然になった。

    // 表示用 .to_f64() チェック
    #[test]
    fn test_to_f64_valid() {
        let measurement = Measurement {
            val: 123456,
            sign: Sign::Plus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        };

        let expected_value = 1234.56; // 小数点位置に合わせた期待値
        assert_eq!(measurement.to_f64(), expected_value);
    }

    #[test]
    fn test_to_f64_negative() {
        let measurement = Measurement {
            val: 123456,
            sign: Sign::Minus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        };

        let expected_value = -1234.56; // 符号がマイナスであることを確認
        assert_eq!(measurement.to_f64(), expected_value);
    }

    #[test]
    fn test_to_f64_all_point_positions() {
        // (PointPosition, Sign, expected)
        let cases = [
            (PointPosition::Zero, Sign::Plus, 123456.0),
            (PointPosition::One, Sign::Plus, 12345.6),
            (PointPosition::Two, Sign::Plus, 1234.56),
            (PointPosition::Three, Sign::Plus, 123.456),
            (PointPosition::Four, Sign::Plus, 12.3456),
            (PointPosition::Five, Sign::Plus, 1.23456),
            (PointPosition::Zero, Sign::Minus, -123456.0),
            (PointPosition::Five, Sign::Minus, -1.23456),
        ];

        for (point, sign, expected) in cases {
            let m = Measurement {
                val: 123456,
                sign,
                point,
                unit: Unit::Mm,
            };
            assert!(
                (m.to_f64() - expected).abs() < 1e-9,
                "point={:?} sign={:?}: got {}, expected {}",
                m.point,
                m.sign,
                m.to_f64(),
                expected
            );
        }
    }
}
