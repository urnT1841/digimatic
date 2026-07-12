//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm (cal_random SinやGaussianはtargetによってはマイナスになりうる)
//!  いろいろなバリエーション対応
//!

use rand::prelude::*;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal as DistNormal};

// WaveGeneratorはドメイン型(Measurement)には触れない。
// f64を返すところまでが責務で、Measurement化はbuild_frame -> parser
// という既存の正規ルート（実機と同じ経路）に任せる。

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
    ///
    /// 戻り値はf64のまま。Measurement化はしない
    /// （呼び出し側で frame_builder::build_frame 等に渡す想定。
    ///  FrameGenerator::generate_value() と同じ責務分担）。
    pub fn next_value(&mut self) -> f64 {
        self.current_step += 1.0;

        // base wave 生成
        let base_val = match self.base {
            // current_step はフィールドの時点で既に f64 なので、
            // ここでの `as f64` は不要（clippy::unnecessary_cast の指摘どおり）。
            BaseWave::Sine => self.amplitude * (self.current_step * 0.1).sin(),
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
                self.random_walk_value += (self.current_step % 3.0) - 1.0;
                self.random_walk_value
            }
        };

        // effector へ流し込む
        // ここで得られる effected が最終的な「Sim測定値(f64, mm)」。
        // Measurement化はしない。呼び出し側で build_frame(effected) のように
        // 渡し、実機と同じ transport -> parser 経路でMeasurementにする。
        self.effect
            .apply_chain(base_val, self.current_step, &mut self.rng)
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

// clippy::new_without_default 対策。中身は `new()` に委譲するだけ。
impl Default for PhysEffect {
    fn default() -> Self {
        Self::new()
    }
}

/// seed付きにも対応した完全ランダム値生成
pub(crate) fn calc_random(rng: &mut StdRng) -> f64 {
    let raw: i32 = rng.random_range(1..=15_000);
    f64::from(raw) / 100.0
}

/// 正弦波（サイン波）計算
///
/// execute_sim.rs から直接呼ばれている公開API。
/// 前回、単一ファイル内だけを見て「未使用」と誤判定し一度削除してしまったが、
/// クレート全体では使用されていたため復元。
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

// calc_quantize は `rg -rn "calc_quantize"` で generator.rs 内の定義行しか
// ヒットしなかった（= 他ファイルからの呼び出しなし）ことを確認済みのため削除。
// Quantize相当のロジックは PhysEffect::apply_chain 内にインライン化されている。
