//!
//! ;

use digimatic::switcher;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // 引数解析
    let mode = digimatic::switcher::parse_args(args).unwrap_or_else(|e| {
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
