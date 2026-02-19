//!
//!  usb-cdc で受信したデータをファイルに保存
//! 

use std::fs::{File, OpenOptions};
use std::time::Duration;
use serialport::SerialPort;
use csv::{Writer, WriterBuilder};

use digimatic::communicator::CdcReceiver;
use digimatic::logger::RxDataLog;

///
/// 
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pico_port_path = digimatic::scanner_of_pico_connection::find_pico_port()?;
    let rx_port = open_pico_port(&pico_port_path)?;
    let mut rx_receiver = CdcReceiver::new(rx_port);

    let mut rx_wtr = create_log_writer("rx_debug.csv")?;
 
    loop {
        // デコードせず、文字列として1行取る
        match rx_receiver.read_str_measurement() {
            Ok(raw) => {
                if let Err(e) = RxDataLog::new(&raw).save_flush(&mut rx_wtr) {
                    eprintln!("Failed to save data: {} ", e)
                }
                println!("Logged: {}", raw);
            }
            Err(e) if CdcReceiver::is_fatal_error(&e) => break, // 切断時のみ終了
            _ => continue,
        }
    }
    Ok(())
}


///
/// portのpathを受け取って Open する
///
fn open_pico_port(path: &str) -> Result<Box<dyn SerialPort>, serialport::Error> {
    let port = serialport::new(path, 115200)
        .timeout(Duration::from_millis(100))
        .open()?;

    Ok(port)
}

///
/// ライター生成
/// 
fn create_log_writer(path: &str) -> Result<Writer<File>, Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    Ok(WriterBuilder::new().has_headers(false).from_writer(file))
}
