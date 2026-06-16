//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm
//!  備考：乱数で生成。実際に送られてくるような近い値が来る的な機能はなし
//!

use rand::prelude::*;
//
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal as DistNormal};
use std::sync::{Mutex, OnceLock};

/// 完全ランダム計算
pub(crate) fn calc_random() -> f64 {
    let raw: i32 = rand::random_range(1..=15_000);
    f64::from(raw) / 100.0
}

/// 正弦波（サイン波）計算
pub(crate) fn calc_sin_wave(
    // AxSin(Θ+δ) を表現。Step_countは外から与える増分
    center: f64,
    amplitude: f64,
    frequency: f64,
    delta: f64,
    step_count: u32,
) -> f64 {
    let theta = f64::from(step_count) * frequency;
    center + amplitude * (theta + delta).sin()
}

/// ガウシアンによるばらつき
/// rand_distr を用いた実装
pub fn calc_gaussian(target: f64, std_dev: f64) -> f64 {
    let normal = DistNormal::new(target, std_dev).expect("std_dev must be positive");

    normal.sample(&mut rand::rng())
}

/// シード固定再現乱数計算
pub(crate) fn calc_seeded_random() -> f64 {
    static RNG_INSTANCE: OnceLock<Mutex<StdRng>> = OnceLock::new();

    let mutex_rng = RNG_INSTANCE.get_or_init(|| {
        let seed: u64 = 2026;
        Mutex::new(StdRng::seed_from_u64(seed))
    });

    let mut guard = mutex_rng.lock().unwrap();
    let rng = &mut *guard;
    let raw: i32 = rng.random_range(1..=15_000);

    f64::from(raw) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_test() {
        for _ in 0..1000 {
            let v = calc_random();

            assert!(v >= 0.01);
            assert!(v <= 150.0);
            assert!(v.is_finite());
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
            assert!(val >= 40.0 && val <= 60.0, "値が範囲外です: {}", val);
        }
    }
}
