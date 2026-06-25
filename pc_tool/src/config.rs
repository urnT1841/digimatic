//! config.rs
//!
//! # Application Configuration Module
//!
//! This module manages the global configuration matrix and runtime policies
//! for the entire Digimatic application suite.
use crate::frame::Unit;
use crate::sim::execute_sim::GenMode;

/// アプリケーションの全般設定
#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub source: DataSource,
    pub ui: UiMode,
    pub format: FrameFormat,
    pub console_mode: ConsoleMode,
    pub gui_config: GuiConfig,
    pub sim_mode: GenMode,
}

impl AppConfig {
    /// 引数解析結果からアプリの動作設定
    pub fn build(source: DataSource, ui: UiMode, is_bin: bool, sim_mode: GenMode) -> Self {
        let format = if is_bin {
            FrameFormat::Bin
        } else {
            FrameFormat::Str
        };
        let console_mode = match ui {
            UiMode::Gui => ConsoleMode::Silent,
            UiMode::Cli => ConsoleMode::Verbose,
        };

        Self {
            source,
            ui,
            format,
            console_mode,
            gui_config: GuiConfig::default(),
            sim_mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    Sim,
    Actual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Cli,
    Gui,
}

/// GUIの表示，スタイル設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuiConfig {
    pub display_unit: Unit,
    pub font_size: f32,
    pub dark_mode: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            display_unit: Unit::Mm,
            font_size: 24.0,
            dark_mode: true,
        }
    }
}

// console 出力設定
#[derive(Debug, Clone, Copy)]
pub enum ConsoleMode {
    Silent,
    Verbose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    Str,
    Bin,
}

/// シリアル通信接続情報保持用 (GUIでの表示用情報ハンドリング)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionInfo {
    pub mode: FrameFormat,
    // あと必要に応じてフィールドを増やす
    // 例えば接続ポート とか それか判明するつながっているマイコンとか
    // pub port_name: String, とか
}

impl ConnectionInfo {
    pub fn new(mode: FrameFormat) -> Self {
        Self { mode }
    }
}
