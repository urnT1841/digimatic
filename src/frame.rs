//!
//! frame.rs
//! 
//! デジマチックのデータフォーマット用の定数や構造体
//! 
//! 

// デジマッチック データフレームの位置
// インデックスだとずれるので
pub const D1: usize  = 0;
pub const D2: usize  = 1;
pub const D3: usize  = 2;
pub const D4: usize  = 3;
pub const D5: usize  = 4;
pub const D6: usize  = 5;
pub const D7: usize  = 6;
pub const D8: usize  = 7;
pub const D9: usize  = 8;
pub const D10: usize = 9;
pub const D11: usize = 10;
pub const D12: usize = 11;
pub const D13: usize = 12;

pub const FRAME_LENGTH:usize = 13;  // デジマチックフレームの長さは13固定


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Sign {
    Plus = 0x00,
    Minus  = 0x08,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Unit {
    Mm = 0x00,
    _Inch = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointPosition {
    Zero = 0x00,    // 000000.
    One = 0x01,     // 00000.0
    Two = 0x02,     // 0000.00
    Three = 0x03,   // 000.000
    Four = 0x04,    // 00.0000
    Five = 0x05,    // 0.00000
}
