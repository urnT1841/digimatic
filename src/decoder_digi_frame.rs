//!
//! digimatic frame のデコーダ
//! 
//! 


use std::io;
use crate::frame::*;

/// 文字列として送られてきたdigimatic frameをデコードする
pub fn decode_digi_frame_string( rx_frame: &str ) -> Result< f64, io::Error >{

    // 受信文字列は /n がついているので除去
    let frame = rx_frame.trim();

    // 構造をタプルに分解してマッチング
    match (
        frame.len(),
        &frame[D1..D4+1],       // ヘッダ
        &frame[D5..D5+1],       // 符号  sign
        &frame[D6..D12],        // 数値
        &frame[D12..D12+1],     // 小数点 point pos
        &frame[D13..D13+1],     // 単位   unit
    ) {
        // 全ての条件が揃った「正解の形」
        (FRAME_LENGTH, "FFFF", s, val_str, p, u) => {            
            let sign = if s == "0" { 1.0 } else { -1.0 };
            let raw_val = val_str.parse::<f64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid digits"))?;
            let pos = u32::from_str_radix(p, 16)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid precision"))?;

            // frameに納められていた計測値とunit まあ日本だとmm (やーぽん法は滅びるべし)
            let mes_data = (raw_val * sign) / 10f64.powi(pos as i32);
            let unit = if u == "0" { "mm" } else { "inch" };

            Ok(mes_data)
        }
        
        // 文字数が違う場合
        (len, _, _, _, _, _) if len != 13 => {
            Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid length: {}", len)))
        }

        // それ以外の「形が違う」場合（ヘッダ違いなど）
        _ => {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid frame format"))
        }
    }
}


// 未実装
// 文字列に変換したものではなく 生のバイナリデータで送られてきたものの復号
pub fn digi_frame_decoder_bin() {

}