//!
//! digimatic データフレームを組み立てる
//! まずは配列でフレームを表見する
//! 
//! 


use crate::frame::*;

const EPSILON:f64 = 1E-5;  // 浮動小数点の揺らぎ対策


pub fn build_frame_array( val   :f64 ) -> [u8; FRAME_LENGTH] {

    let mut digi_frame = [0x0Fu8; FRAME_LENGTH];  //  (0~12の13個 d1->0, d13->12)

    // 下記は固定なので書き換える
    digi_frame[D4] = if val >= 0.0 {Sign::Plus as u8} else { Sign::Minus as u8};     // d5 sign: +:0(0000), -:8(1000) マイナスは来ないけど
    digi_frame[D11] = PointPosition::Two as u8;  // d12 小数点位置は2桁固定
    digi_frame[D12] = Unit::Mm as u8;      // d13 unit 0:mm, 1:inch  mm固定（ミツトヨ純正品の日本品はmmしか返さない(法律上))

    // ここからf64で受け取った測定値を BDCに変換する
    // 小数点以下2桁のxxxx.xx が変換対象
    // 測定値Simは上記を満たしているものしか生成しないので特に注意は払わない
    // 実機(ノギス)からは，いま組み立てようとしているFrameが送られてくるので
    // そのあたりは気にする必要がない。
    // simデータに対してEpsilonを加えたものを整数変換する →  100倍して丸めて，型変換
    // 測定データは d6 - d11(index : 5-10)  の 6つに格納する。
    // 整数変換した値に対して剰余で一番下の桁の数値を手に入れ，10で割って剰余計算から見えなくすることで対応
    // 10の剰余(いわゆるMod)は下の桁からの入手になるので，配列の後ろから詰めることに注意 u8にすることも忘れずに。

    let mut to_bcd = (val.abs()*100.0 + EPSILON).round() as i32;
    for i in (5..=10).rev() {   //.rev() で逆順にする -> 一桁目から入れていく
        digi_frame[i] = (to_bcd % 10) as u8;
        to_bcd /= 10;  //桁ずらす
    }
    digi_frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_frame() {
        let val = 123.456;
        let frame = build_frame_array(val);
        
        // 期待される値をチェック
        assert_eq!(frame[11], PointPosition::Two as u8); // 小数点位置 
        assert_eq!(frame[12], Unit::Mm as u8);           // 単位 
        
        // 123.456 -> 123.46 (四捨五入) -> [0, 1, 2, 3, 4, 6]
        assert_eq!(frame[10], 6); // d11 (1の位) [cite: 59-61]
        
        println!("Test Frame: {:?}", frame);
    }
}