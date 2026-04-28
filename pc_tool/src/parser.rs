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
        return Err(FrameParseError::InvalidBitLength {
            expected: (FRAME_LENGTH * FRAME_NIBBLES),
            found: (bits.len()),
        });
    }

    // msb/lsb 変換準備
    const LSB_MASK: u8 = 0b0001;
    let shifts = match mode {
        BitMode::Msb => [3, 2, 1, 0],
        BitMode::Lsb => [0, 1, 2, 3],
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

/// nibbles: 52要素(13ニブル×4bit)のスライス
/// ここを通ったフレームはデジマチック仕様に沿った正規フレームになる
pub fn validator_bits(nibbles: &[u8]) -> Result<DigimaticFrame, FrameParseError> {
    // 何はともあれ長さチェック
    if nibbles.len() != FRAME_LENGTH {
        return Err(FrameParseError::InvalidBitLength {
            expected: (FRAME_LENGTH),
            found: (nibbles.len()),
        });
    }

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
        data: validate_bcd_slice(data_raw)?, // 固定長チェック済みならOk
        point_pos: PointPosition::try_from(point_raw)?,
        unit: Unit::try_from(unit_raw)?,
    })
}


// データ部分(D6-D11)のBCDチェック (0~9だけ通す)
fn validate_bcd_slice(data: &[u8]) -> Result<[u8; 6], FrameParseError> {
    let mut out = [0u8; 6];

    for (i, &v) in data.iter().enumerate() {
        if v > 9 {
            return Err(FrameParseError::NibbleOutOfRange(v));
        }
        out[i] = v;
    }
    Ok(out)
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

        // nibble変換の char_to_nibbleで実施
        let nibbles: Vec<u8> = frame
            .chars()
            .map(char_to_nibble)
            .collect::<Result<Vec<_>, FrameParseError>>()?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nibbles_should_return_frame() {
        let nibbles = [
            0x0F, 0x0F, 0x0F, 0x0F, // header
            0x00, // sign
            0, 1, 2, 3, 4, 5,    // data
            0x02, // point
            0x00, // unit
        ];

        let frame = validator_bits(&nibbles).unwrap();

        assert_eq!(frame.header, [0x0F; 4]);
        assert_eq!(frame.sign, Sign::Plus);
        assert_eq!(frame.point_pos, PointPosition::Two);
        assert_eq!(frame.unit, Unit::Mm);
    }

    #[test]
    fn except_illigal_length_nibbles() {
        let nibbles = [
            0x0F, 0x0F, 0x0F, 0x0F, // header
            0x00, // sign
            0, 1, 2, 3, 4, 5,    // data
            0x02, // point
            0x00, // unit
            0x10, // extra nibble
        ];

        assert!(validator_bits(&nibbles).is_err());
    }

    // ニブルの各種テスト 正常系と異常系の両方をループで網羅的に
    #[test]
    fn test_invalid_values() {
        let mut nibbles = [
            0x0F, 0x0F, 0x0F, 0x0F, // header
            0x00, // sign (D5)
            0, 1, 2, 3, 4, 5,    // data
            0x02, // point (D12)
            0x00, // unit (D13)
        ];

        // D5: Valid values are 0 and 8
        let valid_d5 = [0x00, 0x08];
        let invalid_d5 = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        ];

        // Check valid D5 values (should pass)
        for &valid in &valid_d5 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D5] = valid;
            assert!(validator_bits(&nibbles_clone).is_ok());
        }

        // Check invalid D5 values (should fail)
        for &invalid in &invalid_d5 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D5] = invalid;
            assert!(validator_bits(&nibbles_clone).is_err());
        }

        // D12: Valid values are 0, 1, 2, 3, 4, 5
        let valid_d12 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let invalid_d12 = [0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];

        // Check valid D12 values (should pass)
        for &valid in &valid_d12 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D12] = valid;
            assert!(validator_bits(&nibbles_clone).is_ok());
        }

        // Check invalid D12 values (should fail)
        for &invalid in &invalid_d12 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D12] = invalid;
            assert!(validator_bits(&nibbles_clone).is_err());
        }

        // D13: Valid values are 0 and 1
        let valid_d13 = [0x00, 0x01];
        let invalid_d13 = [
            0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        ];

        // Check valid D13 values (should pass)
        for &valid in &valid_d13 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D13] = valid;
            assert!(validator_bits(&nibbles_clone).is_ok());
        }

        // Check invalid D13 values (should fail)
        for &invalid in &invalid_d13 {
            let mut nibbles_clone = nibbles.clone();
            nibbles_clone[D13] = invalid;
            assert!(validator_bits(&nibbles_clone).is_err());
        }
    }

    // データ部の異常検出
    #[test]
    fn test_bcd_rejects_invalid_data() {
        let mut nibbles = [
            0x0F,0x0F,0x0F,0x0F,
            0x00,
            1,2,3,4,5,6,
            0x02,
            0x00,
        ];

        nibbles[D7] = 0x0A; // 壊す

        assert!(validator_bits(&nibbles).is_err());
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_string_to_nibbles_conversion() {
            // 変換したい文字列
            let input_str = "FFFF001257720";

            // 文字列からニブルに変換
            let nibbles: Vec<u8> = input_str
                .chars()
                .map(char_to_nibble)
                .collect::<Result<Vec<_>, FrameParseError>>()
                .unwrap();

            // 期待されるニブル値
            let expected_nibbles: Vec<u8> = vec![
                0xF, 0xF, 0xF, 0xF, // FFFF
                0x0, 0x0, // 00
                0x1, 0x2, // 12
                0x5, 0x7, // 57
                0x7, 0x2, // 72
                0x0, // 0
            ];

            // 結果の確認
            assert_eq!(nibbles, expected_nibbles);
        }

        // 追加：ニブルから文字列フレームへの変換テスト
        #[test]
        fn test_nibbles_to_string_conversion() {
            let nibbles: Vec<u8> = vec![
                0xF, 0xF, 0xF, 0xF, // FFFF
                0x0, 0x0, // 00
                0x1, 0x2, // 12
                0x5, 0x7, // 57
                0x7, 0x2, // 72
                0x0, // 0
            ];

            // ニブル列から文字列へ変換
            let result = decode_frame(&nibbles).unwrap();

            // 期待される文字列
            let expected_str = "FFFF001257720".to_string();

            // 結果の確認
            assert_eq!(result, expected_str);
        }
    }

    // 表示用 .to_f64() チェック
    #[test]
    fn test_to_f64_valid() {
        let measurement = Measurement {
            raw_val: "123456".to_string(),
            sign: Sign::Plus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        };

        let expected_value = 1234.56; // 小数点位置に合わせた期待値
        assert_eq!(measurement.to_f64(), expected_value);
    }

    #[test]
    fn test_to_f64_invalid() {
        let measurement = Measurement {
            raw_val: "invalid".to_string(),
            sign: Sign::Plus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        };

        // 無効なデータの場合、NANが返されることを確認
        assert!(measurement.to_f64().is_nan());
    }

    #[test]
    fn test_to_f64_negative() {
        let measurement = Measurement {
            raw_val: "123456".to_string(),
            sign: Sign::Minus,
            point: PointPosition::Two,
            unit: Unit::Mm,
        };

        let expected_value = -1234.56; // 符号がマイナスであることを確認
        assert_eq!(measurement.to_f64(), expected_value);
    }

    // 文字列フレーム→Measurementへの変換検証
    #[test]
    fn test_simulator_string_to_value() {
        let input = "FFFF000945520";
        let frame = DigimaticFrame::try_from(input).unwrap();
        let m = Measurement::try_from(frame).unwrap();
        assert_eq!(m.to_f64(), 94.55);
    }

    // バイナリフレーム(lsb/msb) - >Measurement変換検証
    #[test]
    fn test_binary_to_measurement_flow() {
        // 12.34mm, Plus, Point:2 を模した52ビット(13バイト)の生データ
        // LSB前提: 0x0F は 0b1111 なので、そのまま [1,1,1,1] になる
        let mut bits = Vec::new();
        // Header D1-D4: 0x0F (1111)
        for _ in 0..4 {
            bits.extend_from_slice(&[1, 1, 1, 1]);
        }
        // Sign D5: Plus (0x00)
        bits.extend_from_slice(&[0, 0, 0, 0]);
        // Data D6-D11: 0,0,1,2,3,4 (12.34の一部)
        bits.extend_from_slice(&[0, 0, 0, 0]); // 0
        bits.extend_from_slice(&[0, 0, 0, 0]); // 0
        bits.extend_from_slice(&[1, 0, 0, 0]); // 1 (LSBなら [1,0,0,0])
        bits.extend_from_slice(&[0, 1, 0, 0]); // 2
        bits.extend_from_slice(&[1, 1, 0, 0]); // 3
        bits.extend_from_slice(&[0, 0, 1, 0]); // 4
        // Point D12: 2 (0x02)
        bits.extend_from_slice(&[0, 1, 0, 0]);
        // Unit D13: mm (0x00)
        bits.extend_from_slice(&[0, 0, 0, 0]);

        // 実行
        let nibbles_lsb = parse_bits(&bits, BitMode::Lsb).unwrap();
        let frame_lsb = DigimaticFrame::try_from(&nibbles_lsb[..]).unwrap();
        let m_lsb = Measurement::try_from(frame_lsb).unwrap();
        assert_eq!(m_lsb.to_f64(), 12.34, "LSB failed: got {}", m_lsb.to_f64());

        // 下記のmsbテストは 上で設定しているビット列がmsbとして成り立たない(0~9を超える)ので
        // 今はコメントアウト
        // let nibbles_msb = parse_bits(&bits, BitMode::Msb).unwrap();
        // let frame_msb = DigimaticFrame::try_from(&nibbles_msb[..]).unwrap();
        // let m_msb = Measurement::try_from(frame_msb).unwrap();
        // println!("m_msb: {:?}", m_msb);
        // assert_eq!(m_msb.to_f64(), 12.34, "MSB failed: got {}", m_msb.to_f64());
    }

    #[test]
    fn test_full_chain_decoding_debug() {
        let input = "FFFF000945520";

        // 段階的に実行して、どこが Err なのか突き止める
        let frame_res = DigimaticFrame::try_from(input);
        println!("Step 1 (String -> Frame): {:?}", frame_res);
        let frame = frame_res.expect("Step 1 failed");

        let meas_res = Measurement::try_from(frame);
        println!("Step 2 (Frame -> Measurement): {:?}", meas_res);
        let m = meas_res.expect("Step 2 failed");

        assert_eq!(m.to_f64(), 94.55);
    }

    #[ignore]
    #[test]
    fn debug_print_nibble_conversion_table() {
        println!("\n--- Nibble Maker Conversion Table ---");
        println!("Input Bits | Lsb Mode Result | Msb Mode Result");
        println!("-----------+-----------------+-----------------");

        // 0000 から 1111 まで全てのパターンを試す
        for i in 0..16 {
            let bits = [
                (i >> 3) & 1, // 0番目のビット
                (i >> 2) & 1, // 1番目のビット
                (i >> 1) & 1, // 2番目のビット
                i & 1,        // 3番目のビット
            ];

            // 1ニブル分(4bits)のダミーデータを作成 (13ニブル分必要なので埋める)
            let mut full_bits = vec![0u8; 13 * 4];
            for b in 0..4 {
                full_bits[b] = bits[b];
            }

            let res_lsb = nibble_maker(&full_bits, BitMode::Lsb).unwrap()[0];
            let res_msb = nibble_maker(&full_bits, BitMode::Msb).unwrap()[0];

            println!(
                "  {:?}    |      {:02X} (dec:{:02})   |      {:02X} (dec:{:02})",
                bits, res_lsb, res_lsb, res_msb, res_msb
            );
        }
        println!("---------------------------------------\n");
    }

    #[test]
    fn test_to_f64_all_point_positions() {
        // (PointPosition, Sign, expected)
        let cases = [
            (PointPosition::Zero, Sign::Plus, 123456.0),
            (PointPosition::One, Sign::Plus, 12345.6),
            (PointPosition::Two, Sign::Plus, 1234.56),
            (PointPosition::Three, Sign::Plus, 123.456),
            (PointPosition::Four, Sign::Plus, 12.3456),
            (PointPosition::Five, Sign::Plus, 1.23456),
            (PointPosition::Zero, Sign::Minus, -123456.0),
            (PointPosition::Five, Sign::Minus, -1.23456),
        ];

        for (point, sign, expected) in cases {
            let m = Measurement {
                raw_val: "123456".to_string(),
                sign,
                point,
                unit: Unit::Mm,
            };
            assert!(
                (m.to_f64() - expected).abs() < 1e-9,
                "point={:?} sign={:?}: got {}, expected {}",
                m.point,
                m.sign,
                m.to_f64(),
                expected
            );
        }
    }
}
