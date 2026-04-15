//! parser.rs
//! DigimaticFrame に変換するパーサー

use crate::frame::*;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind};

/// 受け取ったBit列をnibleに変換
/// ここで生成するNibleは msb にして以降はLSB/MSBは意識しないようにする
fn nibble_maker(bits: &[u8], mode: BitMode) -> Result<Vec<u8>, Error> {
    if bits.len() != FRAME_LENGTH * FRAME_NIBBLES {
        return Err(Error::new(ErrorKind::InvalidData, "Insufficient bits"));
    }

    Ok(bits
        .chunks_exact(4)
        .map(|chunk| {
            let chunk: &[u8; 4] = chunk.try_into().unwrap();
            const LSB_MASK: u8 = 0b0001;

            // 物理的なビット順を MSB(0,1,2,3) の重みに正規化
            let shifts = match mode {
                BitMode::Lsb => [3, 2, 1, 0], // LSB-first を 8,4,2,1 の重み
                BitMode::Msb => [0, 1, 2, 3], // Msb-first を 1,2,4,8 の重み
            };

            chunk
                .iter()
                .zip(shifts)
                .fold(0, |acc, (&b, s)| acc | ((b & LSB_MASK) << s))
        })
        .collect())
}

/// bits_buffer: 52要素(13ニブル×4bit)のスライス
pub fn validator_bits(bits_buffer: &[u8], mode: BitMode) -> Result<DigimaticFrame, Error> {
    // 受信日っと列をnibble maker に渡して全ニブルを生成
    let n = nibble_maker(bits_buffer, mode)?;

    // スライス範囲で「意味」を切り出す
    let header_raw = &n[D1..=D3 + 1]; // D1-D4 (4つ)
    let sign_raw = n[D5]; // D5 (1つ)
    let data_raw = &n[D6..=D11]; // D6-D11 (6つ)
    let point_raw = n[D12]; // D12 (1つ)
    let unit_raw = n[D13]; // D13 (1つ)

    // 各パーツを検証
    // まずはHeader
    if header_raw != [0x0F, 0x0F, 0x0F, 0x0F] {
        return Err(Error::new(ErrorKind::InvalidData, "Header mismatch"));
    }

    // のこり組み立て（各型への変換はそれぞれの TryFrom で実施）
    Ok(DigimaticFrame {
        header: header_raw.try_into().unwrap(),
        sign: Sign::try_from(sign_raw)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Sign"))?,
        data: data_raw.try_into().unwrap(),
        point_pos: PointPosition::try_from(point_raw)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Point"))?,
        unit: Unit::try_from(unit_raw)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Unit"))?,
    })
}

/// decode_frame の Rust版 (MSBモードのみ → 文字列に変換)
pub fn decode_frame(nibbles: &[u8]) -> Result<String, ()> {
    if nibbles.len() != 13 {
        return Err(());
    }
    let s: String = nibbles
        .iter()
        .map(|&v| match v {
            15 => 'F',
            10..=14 => (b'A' + v - 10) as char,
            _ => (b'0' + v) as char,
        })
        .collect();
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
            // frame Bit列の長さは 13x4(nibble) = 52Bit
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

// バイナリフレーム → DigimaticFrame
impl TryFrom<(&[u8], BitMode)> for DigimaticFrame {
    type Error = std::io::Error;

    /// input.0: 52bit分のビット列 (各要素は0か1)
    /// input.1: "LSB" or "MSB"
    fn try_from(input: (&[u8], BitMode)) -> Result<Self, Self::Error> {
        let (bits, mode) = input;

        // nibble_makerに渡して MSB nibble として入手
        let n = nibble_maker(bits, mode)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;

        Ok(DigimaticFrame {
            header: n[D1..D5].try_into().unwrap(),
            sign: Sign::try_from(n[D5])
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Sign"))?,
            data: n[D6..D12].try_into().unwrap(),
            point_pos: PointPosition::try_from(n[D12])
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid point pos"))?,
            unit: Unit::try_from(n[D13])
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid Unit"))?,
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
