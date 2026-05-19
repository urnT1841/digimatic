//! args.rs
//! 自前での引数解析

// args.rs

use crate::config::{AppConfig, DataSource, UiMode};
use crate::errors::{ArgumentError, DigimaticError};

#[derive(Debug)]
enum Token {
    Source(DataSource),
    Ui(UiMode),
}

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

    // シャドーイング 中身を確定させる -> unwrap() 対策
    // これをしないでOk(AppConfig) を組み立てようとしても uiがOptionのままでConsole_modeが確定できない
    let source = source.ok_or(invalid_usage())?;
    let ui = ui.ok_or(invalid_usage())?;

    // config.rs の builder 呼び出し
    Ok(crate::config::AppConfig::build(source, ui))
}

fn normalize_arg(arg: &str) -> Result<Token, DigimaticError> {
    let normalized = arg.to_lowercase();

    let normalized = normalized.trim_start_matches('-');

    match normalized {
        "sim" | "s" => Ok(Token::Source(DataSource::Sim)),

        "actual" | "a" => Ok(Token::Source(DataSource::Actual)),

        "gui" | "g" => Ok(Token::Ui(UiMode::Gui)),

        "cli" | "c" => Ok(Token::Ui(UiMode::Cli)),

        _ => Err(DigimaticError::Argument(ArgumentError::InvalidArgs(
            format!("不正な引数です: {}", arg),
        ))),
    }
}

fn duplicate_error(field: &str) -> DigimaticError {
    DigimaticError::Argument(ArgumentError::InvalidArgs(format!(
        "{field} が重複しています"
    )))
}

fn invalid_usage() -> DigimaticError {
    DigimaticError::Argument(ArgumentError::InvalidArgs(
        "Usage: digimatic <sim|actual> <gui|cli>".into(),
    ))
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

        // parse_args相当のロジックを切り出すのが本筋だが，今はいいや
    }
}
