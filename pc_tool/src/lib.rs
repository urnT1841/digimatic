//!
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
//! ## 🛠️ System Architecture and Data Flow
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
//!        ├──① Validate & Decode ──► [decode_raw_data] (Parses Str/Bin formats and checks for errors)
//!        │
//!        ├──② Persist Raw Log   ──► [handle_save_raw_log] (Appends to rx_log.csv)
//!        │
//!        └──③ Dispatch on Success ──► [save_measurement_to_csv] (Appends to measurement.csv)
//!                                 ──► [push_measurement_to_gui] (Streams to GUI via mpsc::Sender)
//! ```
//!
//! ## 📦 Key Modules
//! - `config`: Configuration for application execution modes (GUI/CLI, data sources).
//! - `communicator`: Defines serial port management and the data-reading (`MeasurementRead`) trait.
//! - `received_data_handler`: The core domain engine that validates, parses, and dispatches received raw data.
//! - `dispatcher`: The system entry point that launches the appropriate pipeline loop based on configuration.
//! - `frame`: Data structures defining Digimatic frames and physical quantities (`Measurement`).
//!
//!
//!
//! # Digimatic 受信・パース・配信コアシステム
//!
//! このクレートは、ミツトヨのノギスの出力（やPC/マイコンからのシミュレータ）から送信される
//! 計測データを受信・処理するシステム。
//! 受信データフォーマットはミツトヨの仕様に沿ったStr形式と、生のビット列（Bin形式）に対応。
//! これらのデータを受信・検証・解釈（デコード）した上で、 CSVログへの永続化、GUIへの
//! リアルタイム配信を行うためのコアロジックを提供する。
//!
//! ## 🛠️ システムアーキテクチャ・データフロー
//!
//! データは以下のパイプラインに沿って、上流から下流へ一方向に流れる構造
//!
//! ```text
//! [データソース (Sim / Actual)]
//!        │ (生データ: &[u8])
//!        ▼
//! [共通パイプライン: dispatcher::run_pipeline] ── (無限ループでデータ吸い上げ)
//!        │
//!        ▼
//! [受信データハンドラー: received_data_handler::handle_received_data] ── (全体の管制塔)
//!        │
//!        ├──① デコード・検証 ──► [decode_raw_data] (Str/Bin解析・エラー鑑定)
//!        │
//!        ├──② 生ログ保存   ──► [handle_save_raw_log] (rx_log.csv へ書き込み)
//!        │
//!        └──③ 成功時の配送 ──► [save_measurement_to_csv] (measurement.csv)
//!                           ──► [push_measurement_to_gui] (mpsc::Sender でGUIへ)
//! ```
//!
//! ## 📦 主要モジュール
//! - `config`: アプリケーションの起動モード（GUI/CLI、データソース）を制御する設定情報。
//! - `communicator`: シリアルポートの開閉や、データの読み込み（Reader）トレイトを定義。
//! - `received_data_handler`: 受信した生データの鑑定、パース、および各出力先への配送（本システムのコア）。
//! - `dispatcher`: 起動モード（GUI/CLI）やソースに応じて、適切なパイプラインループを起動するエントリポイント。
//! - `frame`: デジマチックフレームおよび物理量（Measurement）のデータ構造の定義。
//!

pub mod args;
pub mod config;
pub mod dispatcher;
pub mod errors;
pub mod gui_app;
pub mod measurement_history;

pub(crate) mod communicator;
pub(crate) mod frame;
pub(crate) mod logger;
pub(crate) mod parser;
pub(crate) mod presentation;
pub(crate) mod received_data_handler;
pub(crate) mod scanner;
pub(crate) mod sim;
