//! lib.rs
//!
//! # Digimatic Reception, Parsing, and Distribution Core System
//!
//! This crate handles measurement data sent from Mitutoyo calipers (or simulators running on PCs/microcontrollers).
//!
//! It supports the Str format (in accordance with Mitutoyo specifications) and raw bit streams (Bin format) for received data.
//! It provides the core logic to receive, validate, and interpret (decode) this data,
//! then persist it to a CSV log and stream it in real time to the GUI.
//!
//! ## System Architecture and Data Flow
//!
//! Data flows unidirectionally from upstream to downstream along the following pipeline:
//!
//! ```text
//! [Data Source (Sim / Actual)]
//!        │ (Raw Data: &[u8])
//!        ▼
//! [Common Pipeline: dispatcher::run_pipeline] ── (Infinitely loops to read data)
//!        │
//!        ▼
//! [Received Data Handler: received_data_handler::handle_received_data] ── (Central Controller)
//!        │
//!        ├── Validate & Decode ──► [decode_raw_data] (Parses Str/Bin formats and checks for errors)
//!        │
//!        ├── Persist Raw Log   ──► [handle_save_raw_log] (Appends to rx_log.csv)
//!        │
//!        └── Dispatch on Success ──► [save_measurement_to_csv] (Appends to measurement.csv)
//!        │                       ──► [push_measurement_to_gui] (Streams to GUI)
//!                                          │
//!                                          ▼ (MPSC Channel Receiver)
//!                                   [DisplayApp (GUI) / Console]
//!                                          │
//!                                          └──► [MeasurementHistory] (StaticRingBuffer)
//!                                          └──► [egui View (Main/History/Plot(TODO))]
//!
//! ```
//!
//! ## Key Modules
//! - `config`: Configuration for application execution modes (GUI/CLI, data sources).
//! - `communicator`: Defines serial port management and the data-reading (`MeasurementRead`) trait.
//! - `received_data_handler`: The core domain engine that validates, parses, and dispatches received raw data.
//! - `dispatcher`: The system entry point that launches the appropriate pipeline loop based on configuration.
//! - `frame`: Data structures defining Digimatic frames and physical quantities (`Measurement`).
//!

pub mod args;
pub mod config;
pub mod dispatcher;
pub mod errors;
pub mod gui_app;

pub(crate) mod communicator;
pub(crate) mod frame;
pub(crate) mod logger;
pub(crate) mod measurement_history;
pub(crate) mod parser;
pub(crate) mod presentation;
pub(crate) mod received_data_handler;
pub(crate) mod ring_buffer;
pub(crate) mod scanner;
pub(crate) mod sim;
pub(crate) mod web_server;
