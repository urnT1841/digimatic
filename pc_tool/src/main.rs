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
        "actual" => {
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
