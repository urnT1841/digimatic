//! main.rs
//!
//! entry point -> switcher

use digimatic::switcher;

fn main() {
    // 引数解析はswitcher側に委譲する
    let mode = switcher::parse_args().unwrap_or_else(|e| {
        eprintln!("引数エラー: {}", e);
        std::process::exit(1);
    });

    // switcherへ
    if let Err(e) = switcher::run(mode) {
        //  全体で起きたエラーの最終処理
        eprintln!("【システム停止】原因: {}", e);
        std::process::exit(1);
    }
}
