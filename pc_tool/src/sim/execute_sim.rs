//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use std::sync::mpsc::Sender;

use crate::sim::{frame_array_builder, generator};

/// データ生成スレッド
/// channel使ってreceiverに流し込む
pub fn start_generator_thread(tx: Sender<String>) {
    std::thread::spawn(move || {
        loop {
            let val = generator::generator();
            let frame = frame_array_builder::build_frame_array(val);
            let hex: String = frame.iter().map(|b| format!("{:X}", b)).collect();

            // デコード前データ  (debug用)
            // println!("[SIM] gen = {:.3} -> frame={:?}, HEX={:?}", val, frame, hex);

            if tx.send(hex).is_err() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(700));
        }
    });
}
