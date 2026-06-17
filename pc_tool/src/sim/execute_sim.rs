//!
//!  Sim実行
//!  generator -> frame Build -> send -> revice -> display

use std::sync::mpsc::Sender;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::config::FrameFormat;
use crate::frame::TransportFrame;
use crate::sim::frame_builder::build_simulator_payload;
use crate::sim::generator;

/// Simのモード設定
#[derive(Debug, Clone, Copy)]
pub(crate) enum GenMode {
    Random,
    Seed(u64),
    Fixed(f64),
    Gaussian {
        target: f64,
        std_dev: f64,
    },
    SinWave {
        center: f64,    // 振幅のセンター 振幅が 150 (mm) とすると 75mm
        amplitude: f64, // 振幅倍率
        frequency: f64, // 周期 (実装的には角周波数(1ステップあたりの増分角)
        delta: f64,     // 初期位相ずれ
    },
    FaultInjection(FrameSim),
}

/// 将来実装予定の非正規フレーム生成モード設定
// TODO: 直下のひな型と合わせてgenerator実装後に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FrameSim {
    Normal,
    BitFlip { rate: f64 }, // bit反転 引数は確率 0~1 0:壊れない，1: 全部反転  想定は ppmレベル  10^-6 以下
    DropBit,
    ShortPacket,      // ニブル破壊(短い)
    InvalidCharacter, // F 以外の文字 (Strフレーム)
    MissingNewLine,   // 終端文字が来ない  (通信途絶模擬)
}

/// TODO:Generator側の実装が終わってから対応、FrameSim を適用するためのひな形関数
#[allow(dead_code)]
fn apply_frame_sim(sim: FrameSim, frame: &mut Vec<u8>) {
    match sim {
        FrameSim::Normal => {}
        FrameSim::BitFlip { rate: _ } => { /* bit反転 */ }
        FrameSim::DropBit => { /* ビット欠落 */ }
        FrameSim::ShortPacket => { /* truncate */ }
        FrameSim::InvalidCharacter => { /* 文字破壊 */ }
        FrameSim::MissingNewLine => { /* 終端欠落 */ }
    }
}

pub struct FrameGenerator {
    tx: Sender<TransportFrame>,
    format: FrameFormat,
    mode: GenMode,
}

impl FrameGenerator {
    pub fn new(tx: Sender<TransportFrame>, format: FrameFormat, mode: GenMode) -> Self {
        Self { tx, format, mode }
    }

    /// JoinHandleを持っていないのでスレッドないPanicに対応できない
    /// 用途的には気にしすぎ系だが留意するためコメントを残す
    pub fn start_generator_thread(self) {
        std::thread::spawn(move || {
            let mut step: u32 = 0;

            let mut rng = match &self.mode {
                GenMode::Seed(s) => StdRng::seed_from_u64(*s),
                _ => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;
                    StdRng::seed_from_u64(now)
                }
            };

            loop {
                let val = self.generate_value(step, &mut rng);
                let payload = build_simulator_payload(val, self.format);

                if self.tx.send(payload).is_err() {
                    break;
                }

                step = step.wrapping_add(1);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    }

    fn generate_value(&self, step: u32, rng: &mut StdRng) -> f64 {
        match self.mode {
            GenMode::Random => generator::calc_random(rng),
            GenMode::Seed(_) => generator::calc_random(rng),
            GenMode::Fixed(v) => v,
            GenMode::Gaussian { target, std_dev } => generator::calc_gaussian(target, std_dev, rng),
            GenMode::SinWave {
                center,
                amplitude,
                frequency,
                delta,
            } => generator::calc_sin_wave(center, amplitude, frequency, delta, step),

            // TODO:Frame破壊のモードはまだ未実装
            // 暫定としてただの乱数にしておく
            GenMode::FaultInjection(_s) => generator::calc_random(rng),
        }
    }
}
