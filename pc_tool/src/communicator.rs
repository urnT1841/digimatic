//!
//! USB=CDC function
//!   ( cdc:Communication Device Class)
//!

//use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::time::Duration;

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
    pub fn read_str_measurement(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        // read_lineは改行が来るまで待機
        match self.rx_reader.read_line(&mut line) {
            Ok(0) => Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Pico disconnected",
            )),
            Ok(_) => Ok(line.trim().to_string()),
            Err(e) => Err(e),
        }
    }

    // バイナリで送信されたデータ受信 \n終端前提
    pub fn read_bin_measurement(&mut self) -> Result<Vec<u8>, Error> {
        let mut rx_stream = Vec::new();

        match self.rx_reader.read_until(b'\n', &mut rx_stream) {
            Ok(0) => Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Pico disconnected",
            )),
            Ok(n) => {
                if n > 32 {
                    return Err(Error::new(ErrorKind::InvalidData, "Frame too long"));
                }
                // trim_ascii_end を使って末尾を綺麗にする
                Ok(rx_stream.trim_ascii_end().to_vec())
            }
            Err(e) => Err(e),
        }
    }

    pub fn is_fatal_error(err: &std::io::Error) -> bool {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::ConnectionAborted
            | ErrorKind::BrokenPipe
            | ErrorKind::NotFound
            | ErrorKind::PermissionDenied => true,
            _ => false,
        }
    }
}

///
/// pico探す
///
pub const MAX_WAIT_DURATION: Duration = Duration::from_secs(600);

pub fn wait_until_connection() -> Result<String, StopCode> {
    let start_time = std::time::Instant::now();

    loop {
        if let Ok(path) = crate::scanner_of_pico_connection::find_pico_port() {
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
pub fn open_cdc_port(path: &str, baud_rate: u32) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let port = serialport::new(path, BAUD_RATE)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}
