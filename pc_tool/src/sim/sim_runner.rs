//! 'sim_runner.rs'
//!
//! Sim実行制御。
//! 信号生成 -> Frame変換 -> 送信 の流れを管理する。

use std::sync::mpsc::Sender;

use crate::config::FrameFormat;
use crate::frame::TransportFrame;

use crate::sim::frame_builder::build_simulator_payload;
use crate::sim::signal_builder::build_signal_generator;
use crate::sim::sim_config::GenMode;

/// Simデータ生成と送信を管理する
///
/// 実際の値生成はgenerator側へ委譲し、
/// ここでは生成ループと通信処理のみを担当
pub struct FrameGenerator {
    tx: Sender<TransportFrame>,
    format: FrameFormat,
    mode: GenMode,
}

impl FrameGenerator {
    /// 'sim_runner.rs'
    ///
    /// Sim用FrameGeneratorを生成する。
    pub fn new(tx: Sender<TransportFrame>, format: FrameFormat, mode: GenMode) -> Self {
        Self { tx, format, mode }
    }

    /// Sim生成スレッドを開始する
    ///
    /// 処理フロー:
    /// GenMode
    ///  ↓
    /// signal_generator
    ///  ↓
    /// 測定値(f64)
    ///  ↓
    /// TransportFrame
    ///  ↓
    /// channel送信
    pub fn start_generator_thread(self) {
        std::thread::spawn(move || {
            let mut generator = build_signal_generator(self.mode);

            loop {
                let value = generator.next_value();

                let payload = build_simulator_payload(value, self.format);

                if self.tx.send(payload).is_err() {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }
}
