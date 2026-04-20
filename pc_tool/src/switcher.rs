//! 引数から起動モードを選択する

use crate::communicator::SimReceiver;
use crate::errors::{ArgumentError, DigimaticError};
use crate::sim::execute_sim;
use crate::{execute_communicate, gui_main};

#[derive(Debug)]
pub enum AppMode {
    Sim,
    Actual,
    Gui(bool),
}

pub fn run(mode: AppMode) -> Result<(), DigimaticError> {
    match mode {
        AppMode::Sim => {
            // ここでライターを準備
            let rx_wtr = execute_communicate::create_log_writer("rx_log.csv")?;
            let m_wtr = execute_communicate::create_log_writer("measurement.csv")?;

            // 一本化したコア関数を呼ぶ（ライターは Some で、Sender は None）
            execute_sim::run_simulation_core(
                Box::new(SimReceiver::new()), // 受信機
                Some(rx_wtr),                 // 生ログ保存あり
                Some(m_wtr),                  // 測定保存あり
                None,                         // GUI送信なし
            )?;
            Ok(())
        }
        AppMode::Actual => {
            // CLIのみの本番起動
            let (tx, _) = std::sync::mpsc::channel();
            execute_communicate::run_actual_loop(tx).map_err(DigimaticError::from)
        }
        AppMode::Gui(is_sim) => {
            // GUIを起動し、その中で Sim か Actual かを判断する
            gui_main::launch_display(is_sim).map_err(DigimaticError::from)
        }
    }
}

pub fn parse_args() -> Result<AppMode, DigimaticError> {
    let mut args = std::env::args();
    // 一つ目(実行プログラムパス)は読み飛ばす
    args.next();
    // 第1引数を取り出す
    let first_arg = match args.next() {
        None => return Ok(AppMode::Gui(false)), // 引数ないときはGUIモード
        Some(s) => s.trim_start_matches('-').to_lowercase(),
    };

    //引数マッチして飛ばす
    match first_arg.as_str() {
        // CLIシミュレーション
        "sim" | "s" => Ok(AppMode::Sim),
        // CLI実機
        "actual" | "a" => Ok(AppMode::Actual),
        // GUIモード
        "gui" | "g" => {
            let is_sim = args.next().map(|s| s.contains('s')).unwrap_or(false);
            Ok(AppMode::Gui(is_sim))
        }
        _ => Err(DigimaticError::Argument(ArgumentError::InvalidArgs(
            first_arg,
        ))),
    }
}
