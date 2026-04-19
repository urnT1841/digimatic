//! Err type definitions
//!
use serde::Serialize;
use std::io;
use thiserror::Error;

/// communication / IO layer
#[derive(Error, Debug)]
pub enum CommError {
    #[error("IO error")]
    Io(#[from] io::Error),

    #[error("device protocol error: {0}")]
    Protocol(String),

    #[error("Communication channel closed")]
    ChannelClosed,
}

/// parser / validation layer
#[derive(Error, Debug, Serialize, Clone)]
pub enum FrameParseError {
    #[error("invalid bit length: expected {expected}, found {found}")]
    BitLength { expected: usize, found: usize },

    #[error("invalid nibble length: {0}")]
    InvalidBitLength(usize),

    #[error("Incomplete nibble slice: {0}")]
    IncompleteNibble(usize),

    #[error("header mismatch")]
    HeaderMismatch,

    #[error("invalid sign")]
    InvalidSign,

    #[error("invalid point position")]
    InvalidPoint,

    #[error("invalid unit")]
    InvalidUnit,

    #[error("invalid hex char: {0}")]
    InvalidHexChar(char),

    #[error("invalid char: {0}")]
    InvalidChar(char),

    #[error("nibble out of range: {0:#04x}")]
    NibbleOutOfRange(u8),

    #[error("non-ascii input")]
    NonAscii,
}

#[derive(Error, Debug, Clone)]
pub enum ArgumentError {
    // 無効な引数の場合 簡易ヘルプも表示
    #[error("'{0}' は無効な引数です。\n\
                    使用法:\n\
                    (無引数) : GUI起動\n\
                    s(im)    : CLIシミュレーション\n\
                    a(ctual) : CLI実機\n\
                    g(ui) -s : GUIシミュレーション")]
    InvalidArgs(String),

    // 引数が多すぎる場合
    #[error("Too many arguments provided: {0}")]
    TooManyArgs(String),

    // 引数が足りない場合
    #[error("Missing argument: {0}")]
    MissingArgs(String),
}


/// system layer
#[derive(Error, Debug)]
#[error("system error {code}: {message}")]
pub struct SystemError {
    pub code: i32,
    pub message: String,
}

/// Whole App
#[derive(Error, Debug)]
pub enum DigimaticError {
    #[error(transparent)]
    Comm(#[from] CommError),

    #[error(transparent)]
    Parse(#[from] FrameParseError),

    #[error(transparent)]
    System(#[from] SystemError),

    // 引数や設定に関するエラー
    #[error(transparent)]
    Argument(#[from] ArgumentError),

    #[error("GUI error")]
    Gui(#[from] eframe::Error),

}
