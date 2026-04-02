//!
//! digimatic frame の validator
//!
//! 引数：pico (或いはsim::frame builder) から送られてきた フレーム
//!

use crate::frame::*;
use std::io::{Error, ErrorKind};

/// 文字列として送られてきた digimatic frame をパースして DigimaticFrame に変換
/// Measurement構造体に詰めるのをDigimaticFrameに変更
pub fn parse_rx_frame_string(rx_frame: &str) -> Result<DigimaticFrame, Error> {
    // 受信文字列の整形 (Receiverで取り除いているけど念のため) とASCII文字チェック
    let frame = rx_frame.trim();

    if !frame.is_ascii() {
        return Err(Error::new(ErrorKind::InvalidData, "Frame contains non-ASCII characters"));
    }

    // asciiしか来ないけど上でチェックしたうえでバイト列に変換
    let bytes = frame.as_bytes();

    match (
        bytes.len(),
        &bytes[D1..D5],       // ヘッダ
        &bytes[D5..D6],       // 符号
        &bytes[D6..D12],      // 数値
        &bytes[D12..D13],     // 小数点
        &bytes[D13..=D13],     // 単位
    ) {
        (FRAME_LENGTH, b"FFFF", s, val_bytes, p, u) => Ok(DigimaticFrame {
            header: [b'F', b'F', b'F', b'F'],
            sign: convert_sign(s)?,
            data: val_bytes.try_into().unwrap_or([b'0'; 6]),
            point_pos: convert_point(p)?,
            unit: convert_unit(u)?,
        }),
        // 文字数が違う場合
        // 独自の拡張フレームにした場合はここに来るようにする。CRC付加だったり[]でくくるなり。
        (len, _, _, _, _, _) if len != FRAME_LENGTH => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid length: {}", len),
        )),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid frame format")),
    }
}


// 以下は本体側を見やすくするためのヘルパー関数群
// 安全のためバイト列との比較をする  → b"" という形にする
// val parse
fn convert_val(val_bytes: &[u8]) -> Result<String, Error> {
    // 全てのバイトが ASCII 数字かチェック
    if val_bytes.iter().all(|b| b.is_ascii_digit()) {
        // バイト列を文字列に変換（安全にunwrapできる）
        Ok(std::str::from_utf8(val_bytes)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8"))?
            .to_string())
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Invalid numeric data"))
    }
}

// sign parse
fn convert_sign(s: &[u8]) -> Result<Sign, Error> {
    match s {
        b"0" => Ok(Sign::Plus),
        b"8" => Ok(Sign::Minus),
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Unknown sign")),
    }
}

// pointposision parse
fn convert_point(p: &[u8]) -> Result<PointPosition, Error> {
    match p {
        b"0" => Ok(PointPosition::Zero),  // 000000.
        b"1" => Ok(PointPosition::One),   // 00000.0
        b"2" => Ok(PointPosition::Two),   // 0000.00
        b"3" => Ok(PointPosition::Three), // 000.000
        b"4" => Ok(PointPosition::Four),  // 00.0000
        b"5" => Ok(PointPosition::Five),  // 0.00000
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Illigal point")),
    }
}

// unit parse
fn convert_unit(u: &[u8]) -> Result<Unit, Error> {
    match u {
        b"0" => Ok(Unit::Mm),
        b"1" => Ok(Unit::_Inch),
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Unknown unit")),
    }
}

// TODO: 未実装
// 文字列に変換したものではなく 生のバイナリデータで送られてきたものの復号
// 送られてくるのは nibble x 13 の52bit 各nibbleはLSB → ひっくり返してデコード
pub fn parse_rx_frame_bin(rx_stream: &Vec<u8>) {
    let bin_frame = trim_bytes(rx_stream);

 }

// バイナリパーサの下請け関数
pub fn trim_bytes(rx_stream: &Vec<u8>) -> Vec<u8> {
    rx_stream
        .iter()
        .copied()
        .filter(|b| *b != b'\n' && *b != b'\r')
        .collect()
}