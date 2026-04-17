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
}

/// parser / validation layer
#[derive(thiserror::Error, Debug, Serialize, Clone)]
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
}
