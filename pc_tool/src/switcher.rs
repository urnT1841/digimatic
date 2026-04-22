//! 引数から起動モードを選択する
//! switcher.rs

use crate::communicator::SimReceiver;
use crate::errors::{ArgumentError, DigimaticError, FrameParseError};
use crate::execute_communicate;
use crate::frame::{DigimaticFrame, Measurement};
use crate::sim::execute_sim::{run_simulation_core, start_geerator_thread};

#[derive(Debug)]
pub enum AppMode {
    Sim,
    Actual,
    Gui,
}

pub fn run(mode: AppMode) -> Result<(), DigimaticError> {
    // CSVロガー（Sim/Actual共通で使う）
    let rx_wtr = execute_communicate::create_log_writer("rx_log.csv")?;
    let m_wtr = execute_communicate::create_log_writer("measurement.csv")?;

    match mode {
        AppMode::Sim => {
            let (tx, rx) = std::sync::mpsc::channel::<String>();

            // generator → HEX String送信
            start_geerator_thread(tx);

            run_simulation_core(
                Box::new(SimReceiver::new(rx)), // String受信
                Some(rx_wtr),
                Some(m_wtr),
                None, // GUIなし（CLI）
            )?;

            Ok(())
        }
        AppMode::Actual => {
            let (tx, rx) = std::sync::mpsc::channel::<Measurement>();

            execute_communicate::run_actual_loop(tx).map_err(DigimaticError::from)
        }

        AppMode::Gui => {
            unimplemented!("GUIは後で再統合")
            // gui_main::launch_display(rx_gui).map_err(DigimaticError::from)
        }
    }
}

pub fn parse_args() -> Result<AppMode, DigimaticError> {
    let mut args = std::env::args();
    // 一つ目(実行プログラムパス)は読み飛ばす
    args.next();
    // 第1引数を取り出す
    let first_arg = match args.next() {
        None => return Ok(AppMode::Gui), // 引数ないときはGUIモード
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
            Ok(AppMode::Gui)
        }
        _ => Err(DigimaticError::Argument(ArgumentError::InvalidArgs(
            first_arg,
        ))),
    }
}
