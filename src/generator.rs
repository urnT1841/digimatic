//!
//!
//!
//!

use rand::Rng;

pub fn generator() -> f64 {
    // 生成する値は Caliperの測定値を模して 0.02mm ~ 100.0 mm

    let mut rng = rand::rng();
    rng.random_range(0.02..=100.0)
}
