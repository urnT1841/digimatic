//! config.rs
//! アプリケーション全体のぜっていと動作ポリシーの管理

use crate::frame::Unit;

/// アプリケーションの全般設定
#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub source: DataSource,
    pub ui: UiMode,
    pub console_mode: ConsoleMode,
    pub gui_config: GuiConfig,
}

impl AppConfig {
    /// 引数解析結果からアプリのどーさ設定を組み立てる
    pub fn build(source: DataSource, ui: UiMode) -> Self {
        let console_mode = match ui {
            UiMode::Gui => ConsoleMode::Silent,
            UiMode::Cli => ConsoleMode::Verbose,
        };

        Self {
            source,
            ui,
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

/// GUIの表示，スタイル関する設定
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
