//!
//! データレシーバ
//! 
//! 渡されたポートに入ってくるデータを受け取る
//! 
//! 
//! 


use std::io::Read;
use serialport::SerialPort;

pub fn receiver( rx_p: &mut dyn SerialPort ) -> Result<_ , std::io::Error> {

}