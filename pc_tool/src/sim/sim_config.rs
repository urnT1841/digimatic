//! `sim_config.rs`
//!
//! 将来実装予定の非正規フレーム生成モード設定
//! TODO: 直下のひな型と合わせてgenerator実装後に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FrameSim {
    Normal,
    BitFlip { rate: f64 }, // bit反転 引数は確率 0~1 0:壊れない，1: 全部反転  想定は ppmレベル  10^-6 以下
    DropBit,
    ShortPacket,      // ニブル破壊(短い)
    InvalidCharacter, // F 以外の文字 (Strフレーム)
    MissingNewLine,   // 終端文字が来ない  (通信途絶模擬)
}

/// TODO:Generator側の実装が終わってから対応、FrameSim を適用するためのひな形関数
#[allow(dead_code)]
fn apply_frame_sim(sim: FrameSim, frame: &mut Vec<u8>) {
    match sim {
        FrameSim::Normal => {}
        FrameSim::BitFlip { rate: _ } => { /* bit反転 */ }
        FrameSim::DropBit => { /* ビット欠落 */ }
        FrameSim::ShortPacket => { /* truncate */ }
        FrameSim::InvalidCharacter => { /* 文字破壊 */ }
        FrameSim::MissingNewLine => { /* 終端欠落 */ }
    }
}

/// Simのモード設定
#[derive(Debug, Clone, Copy)]
pub enum GenMode {
    Random,
    Seed(u64),
    Fixed(f64),
    Gaussian {
        target: f64,
        std_dev: f64,
    },
    SinWave {
        center: f64,    // 振幅のセンター 振幅が 150 (mm) とすると 75mm
        amplitude: f64, // 振幅倍率
        frequency: f64, // 周期 (実装的には角周波数(1ステップあたりの増分角)
        delta: f64,     // 初期位相ずれ
    },
    FaultInjection(FrameSim),
}
