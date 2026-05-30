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
//! # 計測フレームの構造定義
//!
//! アプリケーション内でパースされた計測データを表現する、コアドメインモデルおよび
//! データ構造を定義する。生のシリアル入力を、型安全な数値・単位・バリデーション
//! メタデータを持つ構造体に変換し保持することで、下流の表示レイヤー（GUI/CLI）に対して
//! 常に整合性の取れたデータを提供。

use crate::errors::FrameParseError;
use crate::received_data_handler::FrameFormat;

// デジマチック データフレームの位置
// インデックスだとずれるので
pub const D1: usize = 0; // header
pub const D4: usize = 3; // header
pub const D5: usize = 4; // sign ( + or - )
pub const D6: usize = 5; // data
pub const D11: usize = 10; // data
pub const D12: usize = 11; // point position
pub const D13: usize = 12; // unit  ( mm or inch )

// 以下のフレーム定義はコード上は未使用 (範囲指定等でスキップされている)
// このcrate(frame.rs)をlib上で pub(crate)扱いにしたためunusedが顕在化
// 将来的にパースロジックをより厳格化する際に復帰，あるいは呼び出し先修正を実施
#[allow(dead_code)]
pub(crate) mod unused_digimatic_frome {
    pub const D2: usize = 1; // header
    pub const D3: usize = 2; // header
    pub const D7: usize = 6; // data
    pub const D8: usize = 7; // data
    pub const D9: usize = 8; // data
    pub const D10: usize = 9; // data
}

// 上記と同じ 改めて対応必要
#[allow(unused_imports)]
pub(crate) use unused_digimatic_frome::*;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    pub val: u32,             // デジマチックフレームの D4-D11
    pub sign: Sign,           // 符号
    pub point: PointPosition, // 小数点位置
    pub unit: Unit,           // 測定値単位 mm ,inch (ただmmしか使わない)
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

/// Measurement構造体の値をf64に変換
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

/// ビット並び順モード
// Msbが送られてくることはないので，これの実装がない。よって未使用のワーニング
// リリースに向けて dead_codeつける。次のバージョンでの扱を検討する
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitMode {
    Lsb,
    Msb,
}

/// 受信フレームを型に押込める
#[derive(Debug, Clone)]
pub enum TransportFrame {
    Str(Vec<u8>),
    Bin(Vec<u8>),
}

impl TransportFrame {
    //中身のバイト列の長さを返す
    pub fn len(&self) -> usize {
        match self {
            TransportFrame::Str(v) => v.len(),
            TransportFrame::Bin(v) => v.len(),
        }
    }

    // 中身のバイト列のスライスを安全に貸す
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            TransportFrame::Str(v) => v,
            TransportFrame::Bin(v) => v,
        }
    }

    // 受信フレームのフォーマット(bin or Str) を返す
    pub fn as_format(&self) -> FrameFormat {
            let format = match self {
                TransportFrame::Str(_) => FrameFormat::Str,
                TransportFrame::Bin(_) => FrameFormat::Bin,
            };
        format
    }
}