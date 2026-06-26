//! `communicator.rs`
//!
//! # USB-CDC Communication Link Layer Module
//!
//! This module establishes and manages the physical USB-CDC serial interface link
//! with the Raspberry Pi Pico hardware, abstracting raw ingestion readers and channel sinks
//! behind a unified dynamic runtime trait interface.
//!

use serialport::SerialPort;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::config::FrameFormat;
use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::frame::{FRAME_LENGTH, TransportFrame};

// PC側からのコード送出は未実装
// 一部(timeoutは使用しているが他は未使用)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StopCode {
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
        // チャネルから最初から生バイト列（Vec<u8>）が届くので、
        //  文字列のパースやトリムは一切不要。そのまま上流へ受け流す
        let payload = self.rx.recv().map_err(|_| CommError::Timeout)?;
        Ok(payload)
    }
}

/// Blocks execution until a valid Raspberry Pi Pico USB/CDC hardware
///    footprint is registered by the OS scanner.
///
/// # Errors
///
/// Returns a [`StopCode::Timeout`] variant if scanning loops exceed
/// the 600-second maximum duration boundary.
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

/// Attempts to claim ownership and open a raw system link to the physical file handle location path.
///
/// # Errors
///
/// Returns a [`DigimaticError::Comm`] wrapper sequence if the OS layer denies connection initialization.
pub const BAUD_RATE: u32 = 115_200;
pub fn open_cdc_port(path: &str, _baud_rate: u32) -> Result<Box<dyn SerialPort>, DigimaticError> {
    let port = serialport::new(path, BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

    Ok(port)
}
