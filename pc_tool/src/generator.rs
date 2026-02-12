//!
//! ノギス測定データっぽい値の出力器
//!
//!  生成範囲：0.01mm ~ 150.00mm
//!  備考：乱数で生成。実際に送られてくるような近い値が来る的な機能はなし
//! 

use rand::Rng;

pub fn generator() -> f64 {
    // 生成する値は Caliperの測定値を模して 0.01mm ~ 150.0 mm

    let mut rng = rand::rng();
    let raw = rng.random_range(1 ..= 150_00);
    let sim_data  = raw as f64 / 100.0;
    sim_data
}
