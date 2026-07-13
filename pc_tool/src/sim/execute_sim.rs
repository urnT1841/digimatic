//!
//!  Sim実行
//!  generator -> frame Build -> send -> revice -> display

use std::sync::mpsc::Sender;

use rand::rngs::StdRng;

use crate::config::FrameFormat;
use crate::frame::TransportFrame;
use crate::sim::frame_builder::build_simulator_payload;
use crate::sim::generator;

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

pub struct FrameGenerator {
    tx: Sender<TransportFrame>,
    format: FrameFormat,
    mode: GenMode,
}

impl FrameGenerator {
    pub fn new(tx: Sender<TransportFrame>, format: FrameFormat, mode: GenMode) -> Self {
        Self { tx, format, mode }
    }

    pub fn start_generator_thread(self) {
        std::thread::spawn(move || {
            // 既存の GenMode から新設計 of WaveGenerator を組み立てる
            use crate::sim::generator::{BaseWave, EffectKind, PhysEffect, WaveGenerator};

            let mut wave_gen = match self.mode {
                GenMode::Random => WaveGenerator::new(BaseWave::Random),
                GenMode::Seed(_) => WaveGenerator::new(BaseWave::Random),
                GenMode::Fixed(v) => WaveGenerator::new(BaseWave::Flat).with_amplitude(v),
                GenMode::Gaussian { target, std_dev } => {
                    let mut effect = PhysEffect::new();
                    effect.active_effects.push(EffectKind::Noise { std_dev });
                    WaveGenerator::new(BaseWave::Flat)
                        .with_amplitude(target)
                        .with_effect(effect)
                }
                GenMode::SinWave { amplitude, .. } => {
                    WaveGenerator::new(BaseWave::Sine).with_amplitude(amplitude)
                }
                GenMode::FaultInjection(_s) => WaveGenerator::new(BaseWave::Random),
            };

            loop {
                // 🌟 変数名を `wave_gen` に変更
                let val = wave_gen.next_value();
                let mut payload = build_simulator_payload(val, self.format);

                if let GenMode::FaultInjection(sim_kind) = self.mode {
                    // apply_frame_sim(sim_kind, &mut payload);
                }

                if self.tx.send(payload).is_err() {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }); // 🌟 スレッドの閉じカッコ
    }

    fn generate_value(&self, step: u32, rng: &mut StdRng) -> f64 {
        match self.mode {
            GenMode::Random => generator::calc_random(rng),
            GenMode::Seed(_) => generator::calc_random(rng),
            GenMode::Fixed(v) => v,
            GenMode::Gaussian { target, std_dev } => generator::calc_gaussian(target, std_dev, rng),
            GenMode::SinWave {
                center,
                amplitude,
                frequency,
                delta,
            } => generator::calc_sin_wave(center, amplitude, frequency, delta, step),

            // TODO:Frame破壊のモードはまだ未実装
            // 暫定としてただの乱数にしておく
            GenMode::FaultInjection(_s) => generator::calc_random(rng),
        }
    }
}
