//! # USB-CDC 通信接続・レシーバ生成モジュール
//! communicator.rs
//!
//! Raspberry Pi Pico（実機）との物理的なシリアル通信（USB-CDC）を確立し、
//! データ受信を行うための「接続管理」および「レシーバオブジェクトの生成」する。
//! Simのデータを受けるレシーバも同様に生成する。
//!
//! ## 主な役割
//! - システムが認識可能な仮想シリアルポートの自動探索・準備
//! - 実機とのシリアルポート接続（ボーレート等の通信設定）の確立
//! - 上流のデータ処理レイヤ（`received_data_handler` 等）へ引き渡すための、通信レシーバの初期化と生成
//!

use serialport::SerialPort;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::frame::{FRAME_LENGTH, TransportFrame};
use crate::received_data_handler::FrameFormat;

// PC側からのコード送出は未実装
// 一部(timeoutは使用しているが他は未使用)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopCode {
    Normal,      // 正常
    Stop,        //
    Timeout,     // 既定の時間Picoが見つからなかった
    HWInterrupt, // 外部の停止ボタン
    HWIssue,     // Picoがノギスを見失ったなど(ノギス取り外したとか)
    UserForce,   // Ctrl-c  (これ捕まえられるの?)
}

#[derive(Debug)]
pub struct CdcReceiver {
    rx_reader: BufReader<Box<dyn SerialPort>>,
    mode: FrameFormat,
}

impl CdcReceiver {
    pub fn new(mut port: Box<dyn SerialPort>, mode: FrameFormat) -> Self {
        port.set_timeout(Duration::from_millis(2000))
            .expect("Failed to set timeout");
        Self {
            rx_reader: BufReader::new(port),
            mode,
        }
    }

    // データ受信
    fn read_raw_frame(&mut self) -> Result<TransportFrame, DigimaticError> {
        match self.mode {
            FrameFormat::Str => {
                let mut rx_stream = Vec::new();

                match self.rx_reader.read_until(b'\n', &mut rx_stream) {
                    Ok(0) => Err(CommError::ConnectionClosed)?,
                    Ok(n) => {
                        if n > 64 {
                            return Err(FrameParseError::InvalidBitLength {
                                expected: (FRAME_LENGTH),
                                found: (n),
                            })?;
                        }
                        let bytes = rx_stream.trim_ascii_end().to_vec();

                        // trim_ascii_end を使って末尾を綺麗にする
                        Ok(TransportFrame::Str(bytes))
                    }
                    Err(e) => Err(CommError::Io(e).into()),
                }
            }
            FrameFormat::Bin => {
                let mut buf = vec![0u8; 13];
                match self.rx_reader.read_exact(&mut buf) {
                    Ok(_) => Ok(TransportFrame::Bin(buf)),
                    // read_exact も Ok(0) 的な事象は Error(UnexpectedEof) 等で返す
                    Err(e) => Err(CommError::Io(e).into()),
                }
            }
        }
    }
}

// これをActual/Simを問わない入力インターフェイスにする
pub trait MeasurementRead: Send {
    fn read_measurement(&mut self) -> Result<TransportFrame, DigimaticError>;
}

// CdcReceiver にトレイトを適用
impl MeasurementRead for CdcReceiver {
    fn read_measurement(&mut self) -> Result<TransportFrame, DigimaticError> {
        self.read_raw_frame()
    }
}

/// Simの時は スレッドで投げられたrxを見に行く
pub struct SimReceiver {
    rx: Receiver<TransportFrame>,
}

impl SimReceiver {
    pub fn new(rx: Receiver<TransportFrame>) -> Self {
        Self { rx }
    }
}

// SimReceiver にトレイトを適用
impl MeasurementRead for SimReceiver {
    fn read_measurement(&mut self) -> Result<TransportFrame, DigimaticError> {
        // 🌟 チャネルから最初から生バイト列（Vec<u8>）が届くので、
        //    文字列のパースやトリムは一切不要。そのまま上流へ右から左へ受け流す！
        let payload = self.rx.recv().map_err(|_| CommError::Timeout)?;
        Ok(payload)
    }
}

///
/// pico探す
///
pub const MAX_WAIT_DURATION: Duration = Duration::from_secs(600);

pub fn wait_until_connection() -> Result<String, StopCode> {
    let start_time = std::time::Instant::now();

    loop {
        if let Ok(path) = crate::scanner::find_pico_port() {
            return Ok(path);
        }
        let elapsed = start_time.elapsed();

        if elapsed > MAX_WAIT_DURATION {
            return Err(StopCode::Timeout);
        }
        print!("\rpicoを探しています。{}秒 ", elapsed.as_secs());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        std::thread::sleep(Duration::from_secs(1));
    }
}

///
/// portのpathを受け取って Open する
///
pub const BAUD_RATE: u32 = 115200;
pub fn open_cdc_port(path: &str, _baud_rate: u32) -> Result<Box<dyn SerialPort>, DigimaticError> {
    let port = serialport::new(path, BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

    Ok(port)
}
