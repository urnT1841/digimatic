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
}

impl CdcReceiver {
    pub fn new(mut port: Box<dyn SerialPort>) -> Self {
        port.set_timeout(Duration::from_millis(2000))
            .expect("Failed to set timeout");
        Self {
            rx_reader: BufReader::new(port),
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

pub trait MeasurementRead {
    fn read_str_measurement(&mut self) -> Result<String, DigimaticError>;
}

// CdcRPeceiver にトレイトを適用
impl MeasurementRead for CdcReceiver {
    fn read_str_measurement(&mut self) -> Result<String, DigimaticError> {
        let mut line = String::new();

        match self.rx_reader.read_line(&mut line) {
            Ok(0) => Err(CommError::ConnectionClosed)?,
            Ok(_) => Ok(line.trim().to_string()),
            Err(e) => Err(CommError::Io(e).into()),
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
pub fn open_cdc_port(
    path: &str,
    _baud_rate: u32,
) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let port = serialport::new(path, BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}
