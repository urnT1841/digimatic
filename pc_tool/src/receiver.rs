//!
//! データレシーバ
//!
//! 渡されたポートに入ってくるデータを受けり文字列に戻して返す
//!
//!
//!

use serialport::SerialPort;

pub fn receiver(rx_p: &mut dyn SerialPort) -> Result<String, std::io::Error> {
    const BUFFER_SIZE: usize = 64;
    let mut read_buf = [0u8; BUFFER_SIZE];

    match rx_p.read(&mut read_buf) {
        // 128bytesまではバッファーにため込む
        Ok(n) => {
            // nは \n を含んだ戻り値なのでこれを使ってスライスする
            let read_byte_line = &read_buf[..n];
            let byte_string = String::from_utf8_lossy(read_byte_line).to_string();
            Ok(byte_string)
        }
        Err(e) => Err(e),
    }
}
