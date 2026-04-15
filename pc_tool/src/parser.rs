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

/// ニブル列 → 文字列フレーム
pub fn decode_frame(nibbles: &[u8]) -> Result<String, Error> {
    if nibbles.len() != FRAME_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid length: {}", nibbles.len()),
        ));
    }
    nibbles.iter().map(|&v| nibble_to_char(v)).collect()
}


/// ニブル値(u8) → ASCII16進文字
fn nibble_to_char(v: u8) -> Result<char, Error> {
    match v {
        0x00..=0x09 => Ok((b'0' + v) as char),
        0x0A..=0x0F => Ok((b'A' + v - 10) as char),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("nibble out of range: {:#04x}", v),
        )),
    }
}

/// ASCII16進文字 → ニブル値(u8)
fn char_to_nibble(c: char) -> Result<u8, Error> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid hex char: '{}'", c),
        )),
    }
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

        if frame.len() != FRAME_LENGTH {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid length: {}", frame.len()),
            ));
        }

        // 全文字をニブル値に変換
        let nibbles: Vec<u8> = frame
            .chars()
            .map(char_to_nibble)
            .collect::<Result<Vec<u8>, _>>()?;

        // header チェック
        if nibbles[D1..=D4] != [0x0F, 0x0F, 0x0F, 0x0F] {
            return Err(Error::new(ErrorKind::InvalidData, "Header mismatch"));
        }

        let to_err = |_| Error::new(ErrorKind::InvalidData, "Invalid enum value");

        Ok(DigimaticFrame {
            header: nibbles[D1..=D4].try_into().unwrap(),
            sign: Sign::try_from(nibbles[D5]).map_err(to_err)?,
            data: nibbles[D6..=D11].try_into().unwrap(),
            point_pos: PointPosition::try_from(nibbles[D12]).map_err(to_err)?,
            unit: Unit::try_from(nibbles[D13]).map_err(to_err)?,
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
        // ニブル値を数字文字に変換して文字列にする
        let raw_val = frame.data
            .iter()
            .map(|&v| nibble_to_char(v))
            .collect::<Result<String, _>>()?;

        Ok(Measurement {
            raw_val,
            sign: frame.sign,
            point: frame.point_pos,
            unit: frame.unit,
        })
    }
}