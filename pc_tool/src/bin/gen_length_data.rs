use rand::Rng;
use std::thread;
use std::time::Duration;

// ライブラリ(digimatic)から必要な機能を呼び出す
use digimatic::port_prepare::port_prepare;
use digimatic::sim::sender::SendMode;
use digimatic::sim::sender::send;

#[derive(Debug, Default)]
struct StatVal {
    count: usize,
    mean: f64,
    max: f64,
    min: f64,
    std_sigma: f64,
}

const MAX_COUNT: usize = 10; // 10個溜まったら終了

fn main() {
    let mut ports = match port_prepare() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ポート準備失敗: {}", e);
            std::process::exit(1);
        }
    };

    let mut rng = rand::rng();
    let mut rnd_vec = Vec::with_capacity(MAX_COUNT);

    println!("統計用データの送信を開始します（全 {} 回）...", MAX_COUNT);

    //  指定回数分、データを送って蓄積する  (send_dataの名残)
    for i in 1..=MAX_COUNT {
        let val: f64 = rng.random_range(0.02..=100.0);
        rnd_vec.push(val);

        // 準備した ports の tx を使って送信
        send(SendMode::SimpleText(val), &mut *ports.tx);
        println!("[{}/{}] 送信中: {:.2}", i, MAX_COUNT, val);

        // 少し待機（シリアル通信の安定のため）
        thread::sleep(Duration::from_millis(100));
    }

    // 生成データの統計値計算
    let stats = cal_stats(&rnd_vec);
    print_stat_res(&stats);
}

// 統計計算用関数
// 他に集計したければこの中にたすか呼び出す形で実装する
fn cal_stats(rvec: &[f64]) -> StatVal {
    let mut s = StatVal::default();
    s.min = f64::MAX;

    if rvec.is_empty() {
        return s;
    }

    let mut sum: f64 = 0.0;
    for &val in rvec {
        s.max = s.max.max(val);
        s.min = s.min.min(val);
        sum += val;
    }
    s.count = rvec.len();
    s.mean = sum / s.count as f64;

    let variance: f64 = rvec.iter().map(|&x| (x - s.mean).powi(2)).sum::<f64>() / rvec.len() as f64;

    s.std_sigma = variance.sqrt();
    s
}

fn print_stat_res(s_result: &StatVal) {
    println!("\n--- 統計結果 ---");
    println!("件数:     {}", s_result.count);
    println!("平均:     {:.3}", s_result.mean);
    println!("最大:     {:.3}", s_result.max);
    println!("最小:     {:.3}", s_result.min);
    println!("標準偏差: {:.3}", s_result.std_sigma);
}
