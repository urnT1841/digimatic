//! # Digimatic Application Binary
//!
//! This is the entry point for the executable.
//! For the core logic, system architecture, and data flow documentation,
//! please refer to the [`digimatic`] library crate root.
//!
//! `main.rs`.
//!
//! entry point -> dispatcher .

use digimatic::{args, dispatcher};
use std::process::exit;

fn main() {
    // 引数解析モジュールからアプリケーション設定を取得
    let mode = args::parse_args().unwrap_or_else(|e| {
        eprintln!("引数エラー: {e}");
        exit(1);
    });

    // dispatcherへ
    if let Err(e) = dispatcher::run(mode) {
        //  App全体で起きたエラーの最終処理
        eprintln!("【システム停止】原因: {e}");
        exit(1);
    }
}
