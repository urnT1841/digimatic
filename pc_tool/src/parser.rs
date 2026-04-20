//! parser.rs
//! DigimaticFrame に変換するパーサー

use crate::errors::FrameParseError;
use crate::frame::*;
use std::convert::TryFrom;

// 物理層から来た信号フレームをnibble_makerに渡すための仲介関数
pub fn parse_bits(bits: &[u8], mode: BitMode) -> Result<[u8; FRAME_LENGTH], FrameParseError> {
    nibble_maker(bits, mode)
}

/// 受け取ったBit列をnibbleに変換
/// ここで生成するNibbleは msb にして以降はLSB/MSBは意識しないようにする
/// この関数は bit列 -> nibble で中身の解釈はしない。なので長さチェックは実施するが
/// そのあとのエラーチェックは行わない (上層のNibble解釈で実施)
fn nibble_maker(bits: &[u8], mode: BitMode) -> Result<[u8; FRAME_LENGTH], FrameParseError> {
    if bits.len() != FRAME_LENGTH * FRAME_NIBBLES {
        return Err(FrameParseError::InvalidBitLength { expected: (FRAME_LENGTH * FRAME_NIBBLES), found: (bits.len()) });
    }

    // msb/lsb 変換準備
    const LSB_MASK: u8 = 0b0001;
    let shifts = match mode {
        BitMode::Lsb => [3, 2, 1, 0],
        BitMode::Msb => [0, 1, 2, 3],
    };

    let mut out = [0u8; FRAME_LENGTH];

    for (i, chunk) in bits.chunks_exact(4).enumerate() {
        let chunk: &[u8; 4] = chunk.try_into().unwrap();

        out[i] = chunk
            .iter()
            .zip(shifts)
            .fold(0, |acc, (&b, s)| acc | ((b & LSB_MASK) << s));
    }
    Ok(out)
}

/// bits_buffer: 52要素(13ニブル×4bit)のスライス
/// ここを通ったフレームはデジマチック仕様に沿った正規フレームになる
pub fn validator_bits(nibbles: &[u8]) -> Result<DigimaticFrame, FrameParseError> {
    // スライス範囲で「意味」を切り出す
    let header_raw = &nibbles[D1..D5]; // D1-D4 (4つ)
    let sign_raw = nibbles[D5]; // D5 (1つ)
    let data_raw = &nibbles[D6..D12]; // D6-D11 (6つ)
    let point_raw = nibbles[D12]; // D12 (1つ)
    let unit_raw = nibbles[D13]; // D13 (1つ)

    // 各パーツを検証
    // まずはHeader
    if header_raw != [0x0F; 4] {
        return Err(FrameParseError::HeaderMismatch);
    }

    // のこり組み立て（各型への変換はそれぞれの TryFrom で実施）
    Ok(DigimaticFrame {
        header: header_raw.try_into().unwrap(), // 固定長チェック済みならOk
        sign: Sign::try_from(sign_raw)?,
        data: data_raw.try_into().unwrap(), // 固定長チェック済みならOk
        point_pos: PointPosition::try_from(point_raw)?,
        unit: Unit::try_from(unit_raw)?,
    })
}

/// ニブル列 → 文字列フレーム
pub fn decode_frame(nibbles: &[u8]) -> Result<String, FrameParseError> {
    if nibbles.len() != FRAME_LENGTH {
        return Err(FrameParseError::IncompleteNibble(nibbles.len()));
    }
    nibbles
        .iter()
        .map(|&v| nibble_to_char(v))
        .collect::<Result<String, _>>()
}

/// ニブル値(u8) → ASCII16進文字
fn nibble_to_char(v: u8) -> Result<char, FrameParseError> {
    match v {
        0x00..=0x09 => Ok((b'0' + v) as char),
        0x0A..=0x0F => Ok((b'A' + v - 10) as char),
        _ => Err(FrameParseError::InvalidChar((b'0' + v) as char)),
    }
}

/// ASCII16進文字 → ニブル値(u8)
fn char_to_nibble(c: char) -> Result<u8, FrameParseError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        _ => Err(FrameParseError::InvalidChar(c)),
    }
}

// バイナリフレーム → DigimaticFrame
// 変換と検証は nibble_maker と validator_bitsに任せて結果だけ受け取る
impl TryFrom<&[u8]> for DigimaticFrame {
    type Error = FrameParseError;

    fn try_from(nibble: &[u8]) -> Result<Self, Self::Error> {
        validator_bits(nibble)
    }
}

/// 文字列フレーム → DigimaticFrame
impl TryFrom<&str> for DigimaticFrame {
    type Error = FrameParseError;

    fn try_from(rx_frame: &str) -> Result<Self, Self::Error> {
        let frame = rx_frame.trim();

        // 長さとASCII チェック入れてあからさまにおかしいのははじく
        if !frame.is_ascii() {
            return Err(FrameParseError::NonAscii);
        }

        if frame.len() != FRAME_LENGTH {
            return Err(FrameParseError::InvalidBitLength { expected: (FRAME_LENGTH), found: (frame.len()) });
        }

        let nibbles: Vec<u8> = frame
            .chars()
            .map(char_to_nibble)
            .collect::<Result<Vec<_>, FrameParseError>>()?;

        // ★ここが本質
        validator_bits(&nibbles)
    }
}

//  Digimatic -> measurement
impl TryFrom<DigimaticFrame> for Measurement {
    type Error = FrameParseError;

    fn try_from(frame: DigimaticFrame) -> Result<Self, Self::Error> {
        // ニブル値を数字文字に変換して文字列にする
        let raw_val = frame
            .data
            .iter()
            .map(|&v| nibble_to_char(v))
            .collect::<Result<String, FrameParseError>>()?;

        Ok(Measurement {
            raw_val,
            sign: frame.sign,
            point: frame.point_pos,
            unit: frame.unit,
        })
    }
}
