//!
//! USB=CDC function
//!   ( cdc:Communication Device Class)
//!

//use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::execute_communicate::FrameFormat;
use crate::frame::FRAME_LENGTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopCode {
    Normal,      // 正常
    Stop,        //
    TimeOut,     // 既定の時間Picoが見つからなかった
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
    pub fn read_raw_frame(&mut self) -> Result<Vec<u8>, DigimaticError> {
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
                        // trim_ascii_end を使って末尾を綺麗にする
                        Ok(rx_stream.trim_ascii_end().to_vec())
                    }
                    Err(e) => Err(CommError::Io(e).into()),
                }
            }
            FrameFormat::Bin => {
                let mut buf = vec![0u8; 13];
                match self.rx_reader.read_exact(&mut buf) {
                    Ok(_) => Ok(buf),
                    // read_exact も Ok(0) 的な事象は Error(UnexpectedEof) 等で返す
                    Err(e) => Err(CommError::Io(e).into()),
                }
            }
        }
    }
}

// これをActual/Simを問わない入力インターフェイスにする
pub trait MeasurementRead: Send {
    fn read_measurement(&mut self) -> Result<Vec<u8>, DigimaticError>;
    fn get_format(&self) -> FrameFormat;
}

// CdcRPeceiver にトレイトを適用
impl MeasurementRead for CdcReceiver {
    fn read_measurement(&mut self) -> Result<Vec<u8>, DigimaticError> {
        self.read_raw_frame()
    }

    fn get_format(&self) -> FrameFormat {
        self.mode
    }
}

/// Simの時は スレッドで投げられたrxを見に行く
pub struct SimReceiver {
    rx: Receiver<String>,
}

impl SimReceiver {
    pub fn new(rx: Receiver<String>) -> Self {
        Self { rx }
    }
}

// SimReceiver にトレイトを適用
impl MeasurementRead for SimReceiver {
    fn read_measurement(&mut self) -> Result<Vec<u8>, DigimaticError> {
        let s = self.rx.recv().map_err(|_| CommError::Timeout)?;
        let trimmed = s.trim();
        Ok(trimmed.as_bytes().to_vec())
    }

    fn get_format(&self) -> FrameFormat {
        FrameFormat::Str
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
            return Err(StopCode::TimeOut);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_receiver_raw_passthrough() {
        let (tx, rx) = std::sync::mpsc::channel();

        // Simから文字列を送る
        let input_str = "FFFF000945520";
        tx.send(input_str.to_string()).unwrap();

        let mut sim = SimReceiver::new(rx);

        //  Vec<u8> にして返す
        let result_bytes = sim.read_measurement().unwrap();

        // 検証：送った文字列がそのままバイト列として届いているか
        assert_eq!(result_bytes, input_str.as_bytes());
    }
}
