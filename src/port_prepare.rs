//!
//!  ユーティリティ port_prepare
//! 
//!  機能：socatを使って送受信用のポートを開く
//!  引数：なし
//!  返値：PortPair構造体
//! 

use std::process::{Child, Command};
use std::io::Error;

#[derive(Debug)]
pub struct PortPair {
    pub source: String,
    pub dist: String,
    pub _socat: Child,
}

pub fn port_prepare() -> Result<PortPair,Error> {
    let s_path = "/tmp/ptty_s";
    let d_path = "/tmp/ptty_d";
    let sp_arg = format!("pty,raw,echo=0,link={}",s_path);
    let dp_arg = format!("pty,raw,echo=0,link={}",d_path);

    // ? 演算子でエラー時に自動で return Err(...)させる
    let child = Command::new("socat")
        .args(["-d", "-d", &sp_arg, &dp_arg])
        .spawn()?;

    Ok(PortPair {
        source: s_path.to_string(),
        dist: d_path.to_string(),
        _socat: child,
    })
}