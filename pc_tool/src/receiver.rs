//! データレシーバ
//!

use serialport::SerialPort;
use std::io::{BufRead, BufReader};

///
/// USB CDC からのデータを読み取る
/// 
/// args:   rx_p :受信ポート
/// return: Result<String,Err>
/// 概要: BufRead,BufReaderを使って \n が来るまで読みこむ
///       読み込んだストリームはResult<String,Err>で返す
/// 
pub fn receiver(rx_p: &mut Box<dyn SerialPort>) -> Result<String, std::io::Error> {
    let mut rx_reader = BufReader::new(rx_p);
    let mut rx_line = String::new();

    match rx_reader.read_line(&mut rx_line) {
        Ok(0) => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "port cloesd")),
        Ok(_) => {
            // \n を取り除いてStringにする
            Ok(rx_line.trim_end().to_string())
        }
        Err(e) => Err(e),
    }
}
