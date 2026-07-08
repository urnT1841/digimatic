//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm (cal_random SinやGaussianはtargetによってはマイナスになりうる)
//!  いろいろなバリエーション対応
//!

use rand::prelude::*;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal as DistNormal};

use crate::frame::Measurement;

/// 排他制御対象の波形定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseWave {
    Flat,
    Random,
    RandomWalk,
    Sine,
    Square,
    // 必要になったらここに追加
}

/// 波形生成部
/// base + parameter + effect
pub struct WaveGenerator {
    base: BaseWave,
    amplitude: f64,
    current_step: f64,
    effect: PhysEffect, // ノイズやらの効果
    random_walk_value: f64,
    rng: StdRng, // 乱数生成用
}

impl WaveGenerator {
    /// コンストラクタ
    pub fn new(base: BaseWave) -> Self {
        Self {
            base,
            amplitude: 10.0, // 振幅初期値
            current_step: 0.0,
            effect: PhysEffect::new(),
            random_walk_value: 0.0, // 初期値はどうするか検討の余地あり。とりあえず0.0
            rng: StdRng::seed_from_u64(2026),
        }
    }

    /// 振幅メソッド
    pub fn with_amplitude(mut self, amp: f64) -> Self {
        self.amplitude = amp;
        self
    }

    /// effectを一括でつなげるメソッドチェーン
    pub fn with_effect(mut self, effect: PhysEffect) -> Self {
        self.effect = effect;
        self
    }

    /// 次の測定値を1件生成して引き出すコアメソッド
    pub fn next_value(&mut self) -> Measurement {
        self.current_step += 1.0;

        // base wave 生成
        let base_val = match self.base {
            BaseWave::Sine => self.amplitude * (self.current_step as f64 * 0.1).sin(),
            BaseWave::Square => {
                if (self.current_step % 20.0) < 10.0 {
                    self.amplitude
                } else {
                    -self.amplitude
                }
            }
            BaseWave::Flat => self.amplitude,
            BaseWave::Random => calc_random(&mut self.rng),
            BaseWave::RandomWalk => {
                self.random_walk_value += (self.current_step % 3.0) as f64 - 1.0;
                self.random_walk_value
            }
        };

        // effector へ流し込む
        let effected = self
            .effect
            .apply_chain(base_val, self.current_step, &mut self.rng);

        Measurement::from_f64(effected)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectKind {
    Noise { std_dev: f64 },
    Quantize { resolution: f64 },
    Drift { speed: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysEffect {
    // 有効な(指定された)効果だけが入る
    pub active_effects: Vec<EffectKind>,
}

impl PhysEffect {
    pub fn new() -> Self {
        Self {
            active_effects: Vec::new(),
        }
    }

    pub fn apply_chain(&self, initial_val: f64, current_step: f64, rng: &mut StdRng) -> f64 {
        // 有効な効果のリストをイテレータ(fold) で回して
        // 順次適用する。これなら順不同で行ける。
        self.active_effects.iter().fold(initial_val, |val, effect| {
            match effect {
                EffectKind::Noise { std_dev } => val + calc_gaussian(val, *std_dev, rng),
                EffectKind::Quantize { resolution } => {
                    if *resolution > 0.0 {
                        (val / resolution).round() * resolution
                    } else {
                        val
                    }
                }
                EffectKind::Drift { speed } => {
                    val + (*speed * current_step) // ステップ数を考慮するなら引数を増やす
                } // 🌟 新しい効果が増えたら、この match の枝（アーム）を増やすだけ！
            }
        })
    }
}

/// seed付きにも対応した完全ランダム値生成
pub(crate) fn calc_random(rng: &mut StdRng) -> f64 {
    let raw: i32 = rng.random_range(1..=15_000);
    f64::from(raw) / 100.0
}

/// 正弦波（サイン波）計算
pub(crate) fn calc_sin_wave(
    // AxSin(Θ+δ) を表現。Step_countは外から与える増分
    center: f64,
    amplitude: f64,
    frequency: f64, // 周波数にしてるけどステップあたりの増分角。用語だと角周波数 相当
    delta: f64,
    step_count: u32,
) -> f64 {
    let theta = f64::from(step_count) * frequency;
    center + amplitude * (theta + delta).sin()
}

/// ガウシアンによるばらつき
/// rand_distr を用いた実装
pub(crate) fn calc_gaussian(target: f64, std_dev: f64, rng: &mut StdRng) -> f64 {
    if let Ok(normal) = DistNormal::new(target, std_dev) {
        normal.sample(rng)
    } else {
        target
    }
}

/// ノギス測定最小値 量子化モデル
pub(crate) fn calc_quantize(value: f64, step: f64) -> f64 {
    (value / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_test() {
        // テスト用に適当なシード（例: 1234）で乱数器を1個用意する
        let mut test_rng = StdRng::seed_from_u64(1234);

        for _ in 0..1000 {
            // 作成した乱数器の参照（&mut test_rng）を渡す
            let v = calc_random(&mut test_rng);

            assert!(v >= 0.01);
            assert!(v <= 150.0);
            assert!(v.is_finite());
        }
    }

    // 種つき乱数（calc_seeded_random）の再現性テスト
    #[test]
    fn test_calc_seeded_random_reproducibility() {
        // 同じシード値で2つの独立した乱数器を作る
        let mut rng1 = StdRng::seed_from_u64(2026);
        let mut rng2 = StdRng::seed_from_u64(2026);

        // 1発目、2発目、3発目……と引いていく数列が「完全に一致」するか検証
        for _ in 0..10 {
            let val1 = calc_random(&mut rng1);
            let val2 = calc_random(&mut rng2);
            assert_eq!(val1, val2, "同じシードなのに値がズレました！");
        }
    }

    #[test]
    fn test_calc_sin_wave_math() {
        // center: 50.0, amplitude: 10.0 のとき、波の範囲は 40.0 ~ 60.0 になるはず
        let center = 50.0;
        let amplitude = 10.0;
        let frequency = 0.1;
        let delta = 0.0;

        // θ+δ が 0 のとき、sin(0) = 0 なので結果は center そのもの
        let val_at_zero = calc_sin_wave(center, amplitude, frequency, delta, 0);
        assert_eq!(val_at_zero, 50.0);

        // 何回かループを回して、計算結果がちゃんと 40.0 から 60.0 の範囲に収まっているか検証
        for step in 0..100 {
            let val = calc_sin_wave(center, amplitude, frequency, delta, step);
            assert!(val >= 40.0 && val <= 60.0, "値が範囲外です: {val}");
        }
    }
}
