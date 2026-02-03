//!
//!  ユーティリティ port_prepare
//! 
//!  機能：socatを使って送受信用のポートを開く
//!  引数：なし
//!  返値：PortPair構造体
//! 

#[derive(Debug)]
struct PortPair {
    pub source: String,
    pub dist: String,
    pub _socat: Child,
}

use std::process::{Child, Command};
use std::io::Error;

pub fn port_prepare() -> Result<PortPair,Error> {
    let sp_arg = "pty,raw,echo=0,link=/tmp/ptty_s";
    let dp_arg = "pty,raw,echo=0,link=/tmp/ptty_d";

    // ? 演算子でエラー時に自動で return Err(...)させる
    let child = Command::new("socat")
        .args(["-d", "-d", sp_arg, dp_arg])
        .spawn()?;

    Ok(PortPair {
        source: sp_arg.to_string(),
        dist: dp_arg.to_string(),
        _socat: child,
    })
}