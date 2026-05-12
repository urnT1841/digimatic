//!
//! USB=CDC function
//!   ( cdc:Communication Device Class)
//!

//use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::errors::{CommError, DigimaticError, FrameParseError};
use crate::execute_communicate::FrameFormat;
use crate::frame::{BitMode, FRAME_LENGTH};
use crate::parser::decode_frame;

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

    // バイナリで送信されたデータ受信 \n終端前提
    pub fn read_bin_measurement(&mut self) -> Result<Vec<u8>, DigimaticError> {
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
}

// これをActual/Simを問わない入力インターフェイスにする
pub trait MeasurementRead {
    fn read_str_measurement(&mut self) -> Result<String, DigimaticError>;
}

// CdcRPeceiver にトレイトを適用
impl MeasurementRead for CdcReceiver {
    fn read_str_measurement(&mut self) -> Result<String, DigimaticError> {
        let mut line = String::new();

        match self.mode {
            FrameFormat::Str => {
                let mut line = String::new();

                match self.rx_reader.read_line(&mut line) {
                    Ok(0) => Err(CommError::ConnectionClosed)?,
                    Ok(_) => Ok(line.trim().to_string()),
                    Err(e) => Err(CommError::Io(e).into()),
                }
            }

            FrameFormat::Bin => {
                let bin = self.read_bin_measurement()?;

                let nibbles = crate::parser::parse_bits(&bin, BitMode::Lsb)?;
                let hex = decode_frame(&nibbles)?;

                Ok(hex)
            }
        }
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
    fn read_str_measurement(&mut self) -> Result<String, DigimaticError> {
        self.rx.recv().map_err(|_| CommError::Timeout.into())
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
        .map_err(|e| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_receiver() {
        let (tx, rx) = std::sync::mpsc::channel();

        tx.send("123.45".to_string()).unwrap();

        let mut sim = SimReceiver::new(rx);
        let result = sim.read_str_measurement().unwrap();

        assert_eq!(result, "123.45");
    }
}
