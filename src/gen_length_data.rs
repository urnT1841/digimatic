//!
//!
//! 
//! 
//! 
//! 
//! 
//! 
//! 


use rand::Rng;

#[derive(Debug,Default)]
struct StatVal {
    count:usize,
    mean:f64,
    max:f64,
    min:f64,
    std_sigma:f64,
}

const MAX_COUNT:usize= 5;

fn main() {

    let mut rng = rand::rng();
    let mut rnd_vec = Vec::with_capacity(MAX_COUNT);

    //乱数生成の確認用
    // for _ in 1..=MAX_COUNT {
    //     // 長さ測定値を生成
    //     let val: f64 = rng.random_range(0.02..=100.0);
    //     rnd_vec.push(val);    
    // }

    // senderに毎秒一つのデータを送る
    // 最終的に送ったデータの統計を表示
    loop {
        let val:f64 = rng.random_range(0.02..=100.0);
        rnd_vec.push(val);
        
        send_data(val);

        if rnd_vec.len() >= 10 {
            break;
        }
    }

    // 生成データの統計値計算
    let stats = cal_stats(&rnd_vec);
    print_stat_res(&stats);

}

///
/// 生成した乱数の統計処理
///
/// 
fn cal_stats ( rvec: &[f64]) -> StatVal {
    let mut s: StatVal = StatVal::default(); // 初期化
    s.min = f64::MAX;

    if rvec.is_empty() { // 受け取ったVecが空なら初期化した統計値構造体を返す
        return s;
    }

    // 統計値算出
    let mut sum:f64 = 0.0;
    for &val in rvec {
        s.max = s.max.max(val);
        s.min = s.min.min(val);
        s.std_sigma = 100.;
        sum += val;
    }
        s.count = rvec.len();
        s.mean = sum / s.count as f64;

    // 分散・標準偏差を求める（2回目のループ：中心化して計算）
    let variance: f64 = rvec.iter()
        .map(|&x| (x - s.mean).powi(2)) // (x - 平均)^2
        .sum::<f64>() / rvec.len() as f64;
    
    s.std_sigma = variance.sqrt(); // 標準偏差確定！

    s

} 


///
///  結果表示
/// 
fn print_stat_res( s_result: &StatVal ) {
    //結果表示
    println!("平均:     {:.3}", s_result.mean);
    println!("最大:     {:.3}", s_result.max);
    println!("最小:     {:.3}", s_result.min);
    println!("標準偏差: {:.3}", s_result.std_sigma);
}
