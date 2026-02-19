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
/// usb-cdcにつながれたPicoを探す。ずーっと待ち続ける。
/// 見つかったらながれてきたぱけっとをろぎんぐする。
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("picoを探しています。");
        // picoを探す
        let pico_port_path = match digimatic::scanner_of_pico_connection::find_pico_port() {
            Ok(path) => path,
            Err(_) => {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        // port open
        let rx_port = match open_pico_port(&pico_port_path) {
            Ok(port) => port,
            Err(_) => continue,
        };

        let mut rx_receiver = CdcReceiver::new(rx_port);
        let mut rx_wtr = create_log_writer("rx_debug.csv")?;
 
        loop {
            match rx_receiver.read_str_measurement() {
                Ok(raw) => {
                    if let Err(e) = RxDataLog::new(&raw).save_flush(&mut rx_wtr) {
                        eprintln!("Failed to save data: {} ", e)
                    }
                    println!("Logged: {}", raw);
                }
                //切断などの致命的な場合，子のループを脱げて外側に出る
                Err(e) if CdcReceiver::is_fatal_error(&e) => break, 
                _ => continue,
            }
        }
    }
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
