//!
//! digimatic frame の validator
//!
//! 引数：pico (或いはsim::frame builder) から送られてきた フレーム
//!

use crate::frame::*;
use std::io::{Error, ErrorKind};

/// 文字列として送られてきたdigimatic frameをパースしてMeasurement構造体に詰め込む
pub fn parse_rx_frame(rx_frame: &str) -> Result<Measurement, Error> {
    // 受信文字列の整形 (Receiverで取り除いているけど念のため) とASCII文字チェック
    let frame = rx_frame.trim();
    
    if !frame.is_ascii() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Frame contains non-ASCII characters",
        ));
    }

    // 構造をタプルに分解してチェック
    // byteに変換するほうが安全
    match (
        frame.len(),
        &frame[D1..D5],       // ヘッダ (D1-D4)
        &frame[D5..D6],       // 符号  sign  D5
        &frame[D6..D12],      // 数値 (D6-D11)
        &frame[D12..D13],     // 小数点 point pos (D12)
        &frame[D13..D13 + 1], // 単位   unit (D13)
    ) {
        // 全ての条件が揃った「正解の形」を Measurement構造体に詰める
        // くどい処理だがシリアル経由の値が相手なので用心側に。とはいえやりすぎ感はある。
        (FRAME_LENGTH, "FFFF", s, val_str, p, u) => Ok(Measurement {
            raw_val: convert_val(val_str)?,
            sign: convert_sign(s)?,
            point: convert_point(p)?,
            unit: convert_unit(u)?,
        }),
        // 文字数が違う場合
        // 独自の拡張フレームにした場合はここに来るようにする。CRC付加だったり[]でくくるなり。
        (len, _, _, _, _, _) if len != FRAME_LENGTH => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid length: {}", len),
        )),

        // それ以外の「形が違う」場合（ヘッダ違いなど）
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid frame format")),
    }
}

// 以下は本体側を見やすくするためのヘルパー関数群
// val parse
fn convert_val(val_str: &str) -> Result<String, Error> {
    // 全ての文字が数字であることを確認(これくらいはやっておく)
    if val_str.chars().all(|c| c.is_ascii_digit()) {
        Ok(val_str.to_string()) // 実体を作って返す
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Invalid numeric data"))
    }
}

// sign parse
fn convert_sign(s: &str) -> Result<Sign, Error> {
    match s {
        "0" => Ok(Sign::Plus),
        "8" => Ok(Sign::Minus),
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Unknown sign")),
    }
}

// pointposision parse
fn convert_point(p: &str) -> Result<PointPosition, Error> {
    match p {
        "0" => Ok(PointPosition::Zero),  // 000000.
        "1" => Ok(PointPosition::One),   // 00000.0
        "2" => Ok(PointPosition::Two),   // 0000.00
        "3" => Ok(PointPosition::Three), // 000.000
        "4" => Ok(PointPosition::Four),  // 00.0000
        "5" => Ok(PointPosition::Five),  // 0.00000
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Illigal point")),
    }
}

// unit parse
fn convert_unit(u: &str) -> Result<Unit, Error> {
    match u {
        "0" => Ok(Unit::Mm),
        "1" => Ok(Unit::_Inch),
        _ => Err(std::io::Error::new(ErrorKind::InvalidData, "Unknown unit")),
    }
}

// 未実装
// 文字列に変換したものではなく 生のバイナリデータで送られてきたものの復号
// pub fn parse_rx_frame_bin(rx_stream: &Vec<u8>) {

// }
