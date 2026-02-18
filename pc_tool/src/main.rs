//!
//! ;
//!
//!
//!

use digimatic::execute_communicate;
use digimatic::sim::execute_sim;

fn main() {

    const MODE: &str = "actual";

    let result = match MODE {
        "sim" => execute_sim::run_simmulation_loop(),
        "actual" => execute_communicate::run_actual_loop(),
        _ => Err("なんかエラー".into()),
    };

    if let Err(e) = result {
        eprintln!("【エラー】システムが停止しました: {}", e);
        std::process::exit(1);
    }
}