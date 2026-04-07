//!
//! ;
//!
//!
//!

use digimatic::execute_communicate;
use digimatic::sim::execute_sim;

#[derive(Debug)]
enum AppMode {
    Sim,
    Actual,
}

// 簡単な引数処理
// sim or actual(ノギスデータ待ち受けモード) defaultはactual
fn parse_args() -> Result<AppMode, String> {
    let args: Vec<String> = std::env::args().collect();
    
    // 1番目の引数がない場合はデフォルトで Actual
    let arg = match args.get(1) {
        Some(s) => s.trim_start_matches('-').to_lowercase(),
        None => return Ok(AppMode::Actual),
    };

    match arg.as_str() {
        "sim" | "s" => Ok(AppMode::Sim),
        "actual" | "a" => Ok(AppMode::Actual),
        _ => Err(format!("未知のモード: '{}'\n使用法: cargo run -- [sim|actual] (s|a)", arg)),
    }
}

fn main() {
    let mode = match parse_args() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}",e);
            std::process::exit(1);
        }
    };
    
    let result = match mode {
        AppMode::Sim => {
            println!("-- Simlation Mode -- ");
            execute_sim::run_simmulation_loop()
        },
        AppMode::Actual => {
            println!("-- Actual Mode  --");
            // guiへの窓口 acutual_loopの引数で必要なので個々で生成
            let (tx, _rx) = std::sync::mpsc::channel::<f64>();
            execute_communicate::run_actual_loop(tx)
        }
    };

    if let Err(e) = result {
        eprintln!("【エラー】システムが停止しました: {}", e);
        std::process::exit(1);
    }
}
