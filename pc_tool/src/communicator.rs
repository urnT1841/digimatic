//!
//! USB=CDC function
//!   ( cdc:Communication Device Class)
//! 

//use serde::Serialize;
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::time::Duration;

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
    pub fn read_bin_measurment(&mut self) -> Result<Vec<u8>, Error> {
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
}
