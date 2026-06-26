//! データレシーバ
//!

use serialport::SerialPort;
use std::io::{self, BufRead, BufReader, Read};
use std::time::Duration;

///
/// USB CDC からのデータを読み取る
/// 
/// args:   rx_p :受信ポート
/// return: Result<String,Err>
/// 概要: BufRead,BufReaderを使って \n が来るまで読みこむ
///       読み込んだストリームはResult<>で返す
/// 
pub fn reciver_string(rx_p: &mut Box<dyn SerialPort>) -> Result<String, std::io::Error> {
    rx_p.set_timeout(Duration::from_millis(100))?; // いつまでだっても終わらないことがないように

    let mut rx_reader = BufReader::new(rx_p);
    let mut rx_line = String::new();

    match rx_reader.read_line(&mut rx_line) {
        Ok(0) => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "port closed")),
        Ok(_) => {
            // \n を取り除いてStringにする
            Ok(rx_line.trim_end().to_string())
        }
        Err(e) => Err(e),
    }
}


/// 
/// USB CDC 受信をバイト列で実施
/// 
/// args:   rx_p :受信ポート
/// return: Result<Vec<u8>,Err>
/// 概要: BufRead,BufReaderを使って \n が来るまで読みこむ
///       読み込んだストリームはResult<>で返す
/// 
pub fn reciver_binary(rx_p: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, std::io::Error> {
    rx_p.set_timeout(Duration::from_millis(100))?; // 無限待ち対策

    let mut rx_reader = BufReader::new(rx_p);
    let mut rx_stream = vec![0x00u8;MAX_SIZE];

    // まずはMAX_SIZEまでため込んで処理
    // max_size は ノイズでごみしか来ない時対策
    match read_until(b'\n', &mut rx_stream) {
        Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "port closed")),
        Ok(n) => {
            let trimed = rx_stream.trim_ascii_end().to_vec();
            Ok(trimed)
        }
        Err(e) => Err(e),
    }
}