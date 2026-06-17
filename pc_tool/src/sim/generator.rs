//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm
//!  いろいろなバリエーション対応
//!

use rand::prelude::*;
use rand_distr::{Distribution, Normal as DistNormal};

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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_test() {
        // テスト用に適当なシード（例: 1234）で乱数器を1個用意する
        let mut test_rng = rand::rngs::StdRng::seed_from_u64(1234);

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
        let mut rng1 = rand::rngs::StdRng::seed_from_u64(2026);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(2026);

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
            assert!(val >= 40.0 && val <= 60.0, "値が範囲外です: {}", val);
        }
    }
}
