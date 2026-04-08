//! 引数から起動モードを選択する

use crate::sim::execute_sim;
use crate::{execute_communicate,gui_main};

#[derive(Debug)]
enum AppMode {
    Sim,
    Actual,
    Gui(bool),
}


pub fn run(mode: AppMode) -> Result<(), String> {
    match mode {
        AppMode::Sim => {
            // CLIのみのシミュレーション起動
            execute_sim::run_simmulation_loop().map_err(|e| e.to_string())
        }
        AppMode::Actual => {
            // CLIのみの本番起動
            let (tx, _) = std::sync::mpsc::channel();
            execute_communicate::run_actual_loop(tx).map_err(|e| e.to_string())
        }
        AppMode::Gui(is_sim) => {
            // GUIを起動し、その中で Sim か Actual かを判断する
            gui_main::launch_display(is_sim).map_err(|e| e.to_string())
        }
    }
}


fn parse_args(args: Vec<String>) -> Result<AppMode, String> {
    // 第1引数（args[1]）を取り出す
    let arg = match args.get(1) {
        // 【案1】引数がないときは、デフォルトで GUI モード（実機）を返す
        None => return Ok(AppMode::Gui(false)), 
        Some(s) => s.trim_start_matches('-').to_lowercase(),
    };

    match arg.as_str() {
        // CLIシミュレーション
        "sim" | "s" => Ok(AppMode::Sim),
        // CLI実機
        "actual" | "a" => Ok(AppMode::Actual),
        // GUIモード（追加の引数でシミュレーションか判定しても面白いです）
        "gui" | "g" => {
            let is_sim = args.get(2)
                .map(|s| s.contains('s'))
                .unwrap_or(false);
            Ok(AppMode::Gui(is_sim))
        },
        _ => Err(format!(
            "'{}' は無効な引数です。\n使用法:\n  (無引数) : GUI起動\n  s(im)    : CLIシミュレーション\n  a(ctual) : CLI実機\n  g(ui) -s : GUIシミュレーション", 
            arg
        )),
    }
}