//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm
//!  備考：乱数で生成。実際に送られてくるような近い値が来る的な機能はなし
//!

use rand::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::{Mutex, OnceLock};

/// 完全ランダム計算
pub(crate) fn calc_random() -> f64 {
    let mut rng = rand::rng();
    let raw: i32 = rng.random_range(1..=15_000);
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

/// リアルノギス（正規分布）計算
pub(crate) fn calc_gaussian(target: f64, std_dev: f64) -> f64 {
    static GAUSSIAN_RNG: OnceLock<Mutex<StdRng>> = OnceLock::new();

    let mutex_rng = GAUSSIAN_RNG.get_or_init(|| {
        // システムの現在時刻（ナノ秒）をシード値にして完全ランダム化
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Mutex::new(StdRng::seed_from_u64(now))
    });

    let mut rng = mutex_rng.lock().unwrap();

    // ボックス＝ミュラー変換
    let u1: f64 = rng.random();
    let u2: f64 = rng.random();

    let u1 = if u1 == 0.0 { 1e-10 } else { u1 };
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

    target + z0 * std_dev
}

/// シード固定再現乱数計算
pub(crate) fn calc_seeded_random() -> f64 {
    static RNG_INSTANCE: OnceLock<Mutex<StdRng>> = OnceLock::new();

    let mutex_rng = RNG_INSTANCE.get_or_init(|| {
        let seed: u64 = 2026;
        Mutex::new(StdRng::seed_from_u64(seed))
    });

    // ロックを確保して、2発目、3発目の乱数を順番に引いていく
    let mut rng = mutex_rng.lock().unwrap();
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
