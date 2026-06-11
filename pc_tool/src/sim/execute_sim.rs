//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use std::sync::mpsc::Sender;

use crate::config::FrameFormat;
use crate::frame::TransportFrame;
use crate::sim::frame_builder::build_simurator_payload;
use crate::sim::generator;

/// Simのモード設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SimMode {
    Random,
    Seed,
    Fixed(f64),
    Gaussian {
        target: f64,
        std_dev: f64,
    },
    SinWave {
        center: f64,    // 振幅のセンター 振幅が 150 (mm) とすると 75mm
        amplitude: f64, // 振幅倍率
        frequency: f64, // 周期
        delta: f64,     // 初期位相ずれ
    },
    FaultInjection(FrameSim),
}

/// 将来実装予定の非正規フレーム生成モード設定
// TODO: 直下のひな型と合わせてgenerator実装後に対応
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
//#[allow(dead_code, unused_variables)] // 未使用の関数と引数の警告を完全に黙らせます
#[allow(dead_code)]
fn apply_frame_sim(sim: FrameSim, frame: &mut Vec<u8>) {
    match sim {
        FrameSim::Normal => {}
        FrameSim::BitFlip { rate } => { /* bit反転 */ }
        FrameSim::DropBit => { /* ビット欠落 */ }
        FrameSim::ShortPacket => { /* truncate */ }
        FrameSim::InvalidCharacter => { /* 文字破壊 */ }
        FrameSim::MissingNewLine => { /* 終端欠落 */ }
    }
}

pub struct FrameGenerator {
    tx: Sender<TransportFrame>,
    format: FrameFormat,
    mode: SimMode,
}

impl FrameGenerator {
    pub fn new(tx: Sender<TransportFrame>, format: FrameFormat, mode: SimMode) -> Self {
        Self { tx, format, mode }
    }

    pub fn start_generator_thred(self) {
        std::thread::spawn(move || {
            loop {
                let val = self.generate_value();
                let sim_payload = build_simurator_payload(val, self.format);

                if self.tx.send(sim_payload).is_err() {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        });
    }

    /// いろんなタイプの計測値生成
    /// ランダムだったり固定だったり
    fn generate_value(&self) -> f64 {
        match self.mode {
            SimMode::Random => generator::calc_random(),
            SimMode::Fixed(v) => {
                // 指定値を出し続ける
                v
            }
            // 構造体スタイルの enum はそのまま変数名を取り出して関数に渡す
            SimMode::Gaussian { target, std_dev } => generator::calc_gaussian(target, std_dev),
            // 名前が同じなら、そのまま generator::sine_wave(min, max, step) 的に渡す
            SimMode::SinWave {
                // $$y = A \sin(\theta + \delta)$$
                center: f64,    // 振幅のセンター 振幅が 150 (mm) とすると 75mm
                amplitude: f64, // 振幅倍率
                frequency: f64, // 周期
                delta: f64,     // 初期位相ずれ
            } => generator::calc_sin_wave(center, amplitude, frequency, delta, step_count),
            // // タプルスタイルの enum は丸括弧 `(sim)` で中身（FrameSim）を取り出す
            SimMode::FaultInjection(sim) => {
                // 異常系モードの時も、ベースとなる数値自体は
                // ひとまず完全ランダムや固定値など、好きな関数から引っ張る
                // （このあと通信フレーム化する段階で、引数の `sim` を使ってビットを壊していく）
                generator()
            }
            SimMode::Seed => 47.67,
        }
    }
}
