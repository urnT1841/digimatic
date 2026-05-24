//! config.rs
//! アプリケーション全体の設定と動作ポリシーの管理

use crate::frame::Unit;
use crate::received_data_handler::FrameFormat;

/// アプリケーションの全般設定
#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub source: DataSource,
    pub ui: UiMode,
    pub format: FrameFormat,
    pub console_mode: ConsoleMode,
    pub gui_config: GuiConfig,
}

impl AppConfig {
    /// 引数解析結果からアプリの動作設定
    pub fn build(source: DataSource, ui: UiMode, is_bin: bool) -> Self {
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
