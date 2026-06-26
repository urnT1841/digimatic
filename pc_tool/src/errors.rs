//! errors.rs
//!
//! # Error Handling and Domain Exception Definitions
//!
//! This module defines the central error hierarchy for the Digimatic tool.
//! It aggregates lower-level errors (such as I/O, serial communication, and frame parsing issues)
//! into a unified, domain-specific `DigimaticError` wrapping enum to ensure type-safe,
//! idiomatic Rust error propagation across the entire pipeline.
//!

use serde::Serialize;
use std::io;
use thiserror::Error;

/// communication / IO layer
#[derive(Error, Debug)]
pub enum CommError {
    #[error("IO error")]
    Io(#[from] io::Error),

    #[error("serial port error")]
    Serial(#[from] serialport::Error),

    #[error("device protocol error: {0}")]
    Protocol(String),

    #[error("connection closed")]
    ConnectionClosed,

    #[error("connection timeout")]
    Timeout,
}

/// parser / validation layer
#[derive(Error, Debug, Serialize, Clone)]
pub enum FrameParseError {
    #[error("invalid bit length: expected {expected}, found {found}")]
    InvalidBitLength { expected: usize, found: usize },

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
    #[error(
        "Error: '{0}' is an invalid argument.\n\n\
                Usage:\n\
                  pc_tool [OPTIONS]...\n\n\
                Options:\n\
                  -g, --gui       Enable Graphical User Interface mode\n\
                  -c, --cli       Run in Headless Command Line Interface mode\n\
                  -s, --sim       Use Software Simulator as data source\n\
                  -a, --actual    Use Actual Mitutoyo device via Serial Port\n\
                  -b, --bin       Enable Debug Mode: print raw binary frames\n\n\
                Examples:\n\
                  pc_tool --gui --sim\n\
                  pc_tool -g -s -b"
    )]
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

impl DigimaticError {
    pub fn is_fatal(&self) -> bool {
        match self {
            DigimaticError::Comm(e) => match e {
                CommError::Timeout => false,
                CommError::Io(_) | CommError::Serial(_) | CommError::ConnectionClosed => true,
                _ => true,
            },
            DigimaticError::Parse(_) => false,
            _ => true,
        }
    }
}
