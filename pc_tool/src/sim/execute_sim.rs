//!
//!  Sim実行
//!  generatar -> frame Build -> send -> revice -> display を
//! すべてRustで実装したもの
//!

use std::sync::mpsc::Sender;

use crate::received_data_handler::FrameFormat;
use crate::sim::frame_builder::build_simurator_payload;
use crate::sim::generator;
use crate::frame::TransportFrame;

/// データ生成スレッド
/// channel使ってreceiverに流し込む
pub fn start_generator_thread(tx: Sender<TransportFrame>, frame_mode: FrameFormat) {
    std::thread::spawn(move || {
        loop {
            let val = generator::generator();

            let sim_payload = build_simurator_payload(val, frame_mode);

            if tx.send(sim_payload).is_err() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(700));
        }
    });
}
