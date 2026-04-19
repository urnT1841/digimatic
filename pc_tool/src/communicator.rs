//!
//! USB=CDC function
//!   ( cdc:Communication Device Class)
//!

//use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::time::Duration;

use crate::errors::{self, CommError, DigimaticError, FrameParseError};

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

    /// 文字列で送信されたデータを1行読み込む
    pub fn read_str_measurement(&mut self) -> Result<String, DigimaticError> {
        let mut line = String::new();
        // read_lineは改行が来るまで待機
        match self.rx_reader.read_line(&mut line) {
            Ok(0) => Err(CommError::ConnectionClosed)?,
            Ok(_) => Ok(line.trim().to_string()),
            Err(e) => Err(CommError::Io(e).into()),
        }
    }

    // バイナリで送信されたデータ受信 \n終端前提
    pub fn read_bin_measurement(&mut self) -> Result<Vec<u8>, DigimaticError> {
        let mut rx_stream = Vec::new();

        match self.rx_reader.read_until(b'\n', &mut rx_stream) {
            Ok(0) => Err(CommError::ConnectionClosed)?,
            Ok(n) => {
                if n > 64 {
                    return Err(FrameParseError::InvalidBitLength(n))?;
                }
                // trim_ascii_end を使って末尾を綺麗にする
                Ok(rx_stream.trim_ascii_end().to_vec())
            }
            Err(e) => Err(CommError::Io(e).into()),
        }
    }

    pub fn is_fatal_error(err: &std::io::Error) -> bool {
        match err.kind() {
            // 継続可能なエラーだけ明示
            ErrorKind::TimedOut | ErrorKind::WouldBlock => false,
            // 未知のエラーも含めて全部致命的
            _ => true,
        }
    }
}

pub trait MeasurementRead {
    fn read_str_measurement(&mut self) -> std::io::Result<String>;
}

// CdcReceiver にトレイトを適用
impl MeasurementRead for CdcReceiver {
    fn read_str_measurement(&mut self) -> std::io::Result<String> {
        self.read_str_measurement()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "digimatic error"))
    }
}

pub struct SimReceiver {
    buffer: std::collections::VecDeque<String>,
}

impl SimReceiver {
    pub fn new() -> Self {
        Self {
            buffer: std::collections::VecDeque::new(),
        }
    }

    pub fn push(&mut self, data: String) {
        self.buffer.push_back(data);
    }

    pub fn read_str_measurement(&mut self) -> Result<String, DigimaticError> {
        match self.buffer.pop_front() {
            Some(line) => Ok(line),
            None => {
                eprintln!("timeout: no data");
                Err(CommError::Timeout.into())
            }
        }
    }
}

// SimReceiver にトレイトを適用
impl MeasurementRead for SimReceiver {
    fn read_str_measurement(&mut self) -> std::io::Result<String> {
        self.read_str_measurement()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "sim error"))
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
