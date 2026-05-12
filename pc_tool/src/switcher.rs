//! 引数から起動モードを選択する
//! switcher.rs

use std::sync::mpsc;

use crate::communicator::CdcReceiver;
use crate::communicator::{MeasurementRead, SimReceiver};
use crate::errors::DigimaticError;
use crate::execute_communicate;
use crate::execute_communicate::handle_received_data;
use crate::frame::Measurement;

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
// モードをハンドリングする構造体
pub struct AppConfig {
    pub source: DataSource,
    pub ui: UiMode,
}

/// エントリポイント
pub fn run(config: AppConfig) -> Result<(), DigimaticError> {
    let input: Box<dyn MeasurementRead> = match config.source {
        DataSource::Sim => {
            // sim用チャンネル作成 -> sim thred生成 → Box詰め
            let (tx_raw, rx_raw) = mpsc::channel();
            crate::sim::execute_sim::start_geerator_thread(tx_raw);
            Box::new(SimReceiver::new(rx_raw))
        }
        DataSource::Actual => {
            let port_path = crate::communicator::wait_until_connection()
                .map_err(|_| DigimaticError::Comm(crate::errors::CommError::ConnectionClosed))?;

            let port = crate::communicator::open_cdc_port(&port_path, 115200)?;

            Box::new(CdcReceiver::new(
                port,
                crate::execute_communicate::FrameFormat::Str,
            ))
        }
    };

    //ここにuiモード分け
    match config.ui {
        UiMode::Gui => {
            let (tx_gui, rx_gui) = mpsc::channel();
            // パイプラインを別スレッドで起動
            // inputの所有権をスレッド内に移動させる
            std::thread::spawn(move || {
                if let Err(e) = run_pipeline(input, Some(tx_gui)) {
                    eprintln!("[Error] Pipeline failde: {:?}", e);
                }
            });
            // メインスレッドでGUIを起動（rx_guiからデータ受け取れる)
            crate::gui_main::launch_display(rx_gui).map_err(DigimaticError::from)
        }
        UiMode::Cli => {
            // cliの時はメインスレッドで直接パイプラン実行
            // txは不要 → Noneにしておく
            run_pipeline(input, None)
        }
    }
}

enum Token {
    Source(DataSource),
    Ui(UiMode),
}

// 引数解析
pub fn parse_args() -> Result<AppConfig, DigimaticError> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() != 2 {
        return Err(invalid_usage());
    }

    let mut source = None;
    let mut ui = None;

    for arg in args {
        match normalize_arg(&arg)? {
            Token::Source(s) => {
                if source.is_some() {
                    return Err(duplicate_error("source"));
                }
                source = Some(s);
            }

            Token::Ui(u) => {
                if ui.is_some() {
                    return Err(duplicate_error("ui"));
                }
                ui = Some(u);
            }
        }
    }

    let source = source.ok_or_else(|| invalid_usage())?;
    let ui = ui.ok_or_else(|| invalid_usage())?;

    Ok(AppConfig { source, ui })
}

/// 引数を検証する。呼び出しもとには正しい引数だった場合その頭文字を返す(aとかgとか)
/// 引数増やす場合はここをいじる
fn normalize_arg(arg: &str) -> Result<Token, DigimaticError> {
    let normalized = arg.to_lowercase().trim_start_matches('-').to_string();

    match normalized.as_str() {
        "sim" | "s" => Ok(Token::Source(DataSource::Sim)),
        "actual" | "a" => Ok(Token::Source(DataSource::Actual)),

        "gui" | "g" => Ok(Token::Ui(UiMode::Gui)),
        "cli" | "c" => Ok(Token::Ui(UiMode::Cli)),

        _ => Err(DigimaticError::Argument(
            crate::errors::ArgumentError::InvalidArgs(format!("不正な引数です: {}", arg)),
        )),
    }
}

fn duplicate_error(field: &str) -> DigimaticError {
    DigimaticError::Argument(crate::errors::ArgumentError::InvalidArgs(format!(
        "{} が重複しています",
        field
    )))
}

fn invalid_usage() -> DigimaticError {
    DigimaticError::Argument(crate::errors::ArgumentError::InvalidArgs(
        "Usage: digimatic <sim|actual> <gui|cli>".into(),
    ))
}

// 共通ループ
pub fn run_pipeline(
    mut input: Box<dyn MeasurementRead>,
    tx: Option<mpsc::Sender<Measurement>>,
) -> Result<(), DigimaticError> {
    let mut rx_wtr = Some(execute_communicate::create_log_writer("rx_log.csv")?);
    let mut m_wtr = Some(execute_communicate::create_log_writer("measurement.csv")?);

    loop {
        // data受信
        let data = input.read_str_measurement()?;
        if data.is_empty() {
            continue;
        }

        // 共通ハンドラ処理
        handle_received_data(&data, &mut rx_wtr, &mut m_wtr, &tx)?;

        if tx.is_none() {
            // cli modeの時のコンソールへの表示など。下記はダミー
            print!("実行中");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_arg_source() {
        let r = normalize_arg("sim").unwrap();
        match r {
            Token::Source(DataSource::Sim) => {}
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_normalize_arg_ui() {
        let r = normalize_arg("gui").unwrap();
        match r {
            Token::Ui(UiMode::Gui) => {}
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn test_invalid_arg() {
        assert!(normalize_arg("xxx").is_err());
    }

    #[test]
    fn test_duplicate_detection() {
        let args = vec!["sim".to_string(), "sim".to_string()];

        // parse_args相当のロジックを切り出すのが理想
    }
}
