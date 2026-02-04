//!
//!  ユーティリティ port_prepare
//!
//!  機能：socatを使って送受信用のポートを開く
//!  引数：なし
//!  返値：PortPair構造体
//!

use serialport::SerialPort;
use std::io::Error;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct PortPair {
    pub tx_path: String,
    pub rx_path: String,
    pub tx: Box<dyn SerialPort>, // 送信用ポート
    pub rx: Box<dyn SerialPort>, // 受信用ポート
    pub _socat: Child,
}

pub fn port_prepare() -> Result<PortPair, Error> {
    let tx_path = "/tmp/ptty_s";
    let rx_path = "/tmp/ptty_d";
    let tx_arg = format!("pty,raw,echo=0,link={}", tx_path);
    let rx_arg = format!("pty,raw,echo=0,link={}", rx_path);

    // ? 演算子でエラー時に自動で return Err(...)させる
    let child = Command::new("socat")
        .args(["-d", "-d", &tx_arg, &rx_arg])
        .spawn()?;

    // socatの作業待ち (時間待ち。べつにsocatの出力を監視しているわけではないので注意)
    thread::sleep(Duration::from_millis(200));

    // tx,rx のそれぞれのポートを開く
    let tx = serialport::new(tx_path, 9600)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    let rx = serialport::new(rx_path, 9600)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    Ok(PortPair {
        tx_path: tx_path.to_string(),
        rx_path: rx_path.to_string(),
        tx,
        rx,
        _socat: child,
    })
}
