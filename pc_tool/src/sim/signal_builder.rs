//! `signal_builder`
//!
//! Sim設定から信号生成器を構築する。
//! GenModeとWaveGeneratorの橋渡しを担当する。

use crate::sim::generator::{BaseWave, EffectKind, PhysEffect, WaveGenerator};
use crate::sim::sim_config::GenMode;

/// Sim設定からWaveGeneratorを生成する。
///
/// GenModeの内容に応じて、
/// 使用する波形とEffectを組み合わせる。
pub fn build_signal_generator(mode: GenMode) -> WaveGenerator {
    match mode {
        GenMode::Random => WaveGenerator::new(BaseWave::Random),
        GenMode::Seed(_) => WaveGenerator::new(BaseWave::Random),
        GenMode::Fixed(value) => WaveGenerator::new(BaseWave::Flat).with_amplitude(value),
        GenMode::Gaussian { target, std_dev } => {
            let effect = PhysEffect::new().with_effect(EffectKind::Noise { std_dev });

            WaveGenerator::new(BaseWave::Flat)
                .with_amplitude(target)
                .with_effect(effect)
        }
        GenMode::SinWave { amplitude, .. } => {
            WaveGenerator::new(BaseWave::Sine).with_amplitude(amplitude)
        }
        GenMode::FaultInjection(_) => WaveGenerator::new(BaseWave::Random),
    }
}
