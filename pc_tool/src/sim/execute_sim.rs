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
        step_count: u32,
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
            // SinWave のときだけインクリメントが必要なので最初は None に設定
            let mut current_step: Option<u32> = match self.mode {
                SimMode::SinWave { step_count, .. } => Some(step_count),
                _ => None,
            };

            loop {
                //現在のステップ数(あれば)を渡して値を生成
                let val = self.generate_value(current_step.unwrap_or(0));

                let sim_payload = build_simurator_payload(val, self.format);

                if self.tx.send(sim_payload).is_err() {
                    break;
                }

                // SinWave モード →  +1 インクリメント
                if let Some(ref mut step) = current_step {
                    *step = step.wrapping_add(1);
                }

                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        });
    }

    fn generate_value(&self, step_count: u32) -> f64 {
        match self.mode {
            SimMode::Random => generator::calc_random(),
            SimMode::Fixed(v) => v,
            SimMode::Gaussian { target, std_dev } => generator::calc_gaussian(target, std_dev),

            // enum 内の初期設定値（center や amplitude）と、スレッド側で管理している step_count を組み合わせる
            SimMode::SinWave {
                center,
                amplitude,
                frequency,
                delta,
                .. // enum 内の step_count は無視して、引数の最新の step_count を使う
            } => generator::calc_sin_wave(center, amplitude, frequency, delta, step_count),

            SimMode::FaultInjection(_sim) => generator::calc_random(),
            SimMode::Seed => generator::calc_seeded_random(),
        }
    }
}
