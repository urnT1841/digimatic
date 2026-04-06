//! parser.rs
//! DigimaticFrame に変換するパーサー

use crate::frame::*;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

/// nibble_maker の Rust版
fn nibble_maker(bits: &[u8; 4], mode: BitMode) -> u8 {
    match mode {
        BitMode::Lsb => (bits[0] << 3) | (bits[1] << 2) | (bits[2] << 1) | bits[3],
        BitMode::Msb => bits[0] | (bits[1] << 1) | (bits[2] << 2) | (bits[3] << 3),
    }
}

/// validator の Rust版
/// bits_buffer: 52要素(13ニブル×4bit)のスライス
pub fn validator_bits(bits_buffer: &[u8], mode: BitMode) -> Option<Vec<u8>> {
    let bits: &[u8; 52] = bits_buffer.try_into().ok()?;

    (0..13)
        .map(|i| {
            let nibble_bits: &[u8; 4] = bits[i * 4..(i + 1) * 4].try_into().ok()?;
            let val = nibble_maker(nibble_bits, mode);

            let valid = match (i, mode) {
                (0..=3, _)         => val == 0xF,
                (4, BitMode::Msb)  => matches!(val, 0 | 8),
                (4, BitMode::Lsb)  => matches!(val, 0 | 1),
                (5..=10, BitMode::Msb) => val <= 9,
                (5..=10, BitMode::Lsb) => [0u8, 8, 4, 12, 2, 10, 6, 14, 1, 9].contains(&val),
                (11, BitMode::Msb) => val <= 5,
                (11, BitMode::Lsb) => [0u8, 8, 4, 12, 2, 10].contains(&val),
                (12, BitMode::Msb) => matches!(val, 0 | 1),
                (12, BitMode::Lsb) => matches!(val, 0 | 8),
                _ => false,
            };

            valid.then_some(val)
        })
        .collect()
}

/// decode_frame の Rust版 (MSBモードのみ → 文字列に変換)
pub fn decode_frame(nibbles: &[u8]) -> Result<String, ()> {
    if nibbles.len() != 13 {
        return Err(());
    }
    let s: String = nibbles.iter().map(|&v| {
        match v {
            15 => 'F',
            10..=14 => (b'A' + v - 10) as char,
            _ => (b'0' + v) as char,
        }
    }).collect();
    Ok(s)
}

/// 文字列フレーム → DigimaticFrame
impl TryFrom<&str> for DigimaticFrame {
    type Error = std::io::Error;

    fn try_from(rx_frame: &str) -> Result<Self, Self::Error> {
        let frame = rx_frame.trim();

        if !frame.is_ascii() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Frame contains non-ASCII",
            ));
        }
        let bytes = frame.as_bytes();

        if bytes.len() != FRAME_LENGTH {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid length: {}", bytes.len()),
            ));
        }

        Ok(DigimaticFrame {
            header: bytes[D1..D5].try_into().unwrap(),
            sign: convert_sign(&bytes[D5..D6])?,
            data: bytes[D6..D12].try_into().unwrap(),
            point_pos: convert_point(&bytes[D12..D13])?,
            unit: convert_unit(&bytes[D13..D13 + 1])?,
        })
    }
}

/// helper functions
fn convert_sign(s: &[u8]) -> Result<Sign, Error> {
    match s {
        b"0" => Ok(Sign::Plus),
        b"8" => Ok(Sign::Minus),
        _ => Err(Error::new(ErrorKind::InvalidData, "Unknown sign")),
    }
}

fn convert_point(p: &[u8]) -> Result<PointPosition, Error> {
    match p {
        b"0" => Ok(PointPosition::Zero),
        b"1" => Ok(PointPosition::One),
        b"2" => Ok(PointPosition::Two),
        b"3" => Ok(PointPosition::Three),
        b"4" => Ok(PointPosition::Four),
        b"5" => Ok(PointPosition::Five),
        _ => Err(Error::new(ErrorKind::InvalidData, "Illegal point")),
    }
}

fn convert_unit(u: &[u8]) -> Result<Unit, Error> {
    match u {
        b"0" => Ok(Unit::Mm),
        b"1" => Ok(Unit::_Inch),
        _ => Err(Error::new(ErrorKind::InvalidData, "Unknown unit")),
    }
}

// バイナリフレーム → DigimaticFrame
impl TryFrom<(&[u8], BitMode)> for DigimaticFrame {
    type Error = std::io::Error;

    /// input.0: 52bit分のビット列 (各要素は0か1)
    /// input.1: "LSB" or "MSB"
    fn try_from(input: (&[u8], BitMode)) -> Result<Self, Self::Error> {
        let (bits, mode) = input;

        // 52bit必要
        if bits.len() != 52 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid binary frame length: {}", bits.len()),
            ));
        }

        let validated = validator_bits(bits, mode)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Validation failed"))?;

        // MSB文字列に変換してパース
        let decoded_str = decode_frame(&validated)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Decode failed"))?;
        let ascii_bytes = decoded_str.as_bytes();

        Ok(DigimaticFrame {
            header: ascii_bytes[D1..D5].try_into().unwrap(),
            sign: convert_sign(&ascii_bytes[D5..D6])?,
            data: ascii_bytes[D6..D12].try_into().unwrap(),
            point_pos: convert_point(&ascii_bytes[D12..D13])?,
            unit: convert_unit(&ascii_bytes[D13..FRAME_LENGTH])?,
        })
    }
}


//  Digimatic -> measurement
impl TryFrom<DigimaticFrame> for Measurement {
    type Error = std::io::Error;

    fn try_from(frame: DigimaticFrame) -> Result<Self, Self::Error> {
        let raw_val = std::str::from_utf8(&frame.data)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            .to_string();

        Ok(Measurement {
            raw_val,
            sign: frame.sign,
            point: frame.point_pos,
            unit: frame.unit,
        })
    }
}
