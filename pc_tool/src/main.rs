//!
//! ;
//!
//!
//!

use digimatic::execute_communicate;
use digimatic::sim::execute_sim;
use std::env;

fn main() {
    // 簡単な引数処理 sim or actual(default)
    let args:Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("actual");
    
    let result = match mode {
        "sim" => {
            println!("-- Simlation Mode -- ");
            execute_sim::run_simmulation_loop()
        },
        "actual" => {
            println!("-- Actual Mode  --");
            // guiへの窓口 acutual_loopの引数で必要なので個々で生成
            let (tx, _rx) = std::sync::mpsc::channel::<f64>();
            execute_communicate::run_actual_loop(tx)
        }
        _ => Err("なんかエラー".into()),
    };

    if let Err(e) = result {
        eprintln!("【エラー】システムが停止しました: {}", e);
        std::process::exit(1);
    }
}
