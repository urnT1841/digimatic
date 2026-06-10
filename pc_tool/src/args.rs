//! `args.rs`
//!
//! # Argument Parsing Module
//!
//! This module parses command-line arguments (`std::env::args`) at application startup
//! to determine the execution modes and configurations.
//!
//! ## Key Responsibilities
//! - Parsing option strings (e.g., `--sim`, `--gui`, `-b`) into internal tokens.
//! - Enforcing validation guards to reject duplicate flags or invalid combinations.
//! - Constructing the safe `AppConfig` configuration matrix.
//!
//! ## Safety and Error Handling
//! As the untrusted input boundary of the application, this module guarantees runtime safety
//! by intercepting invalid arguments immediately and translating them into domain-specific error types.

use crate::config::{AppConfig, DataSource, UiMode};
use crate::errors::{ArgumentError, DigimaticError};

#[derive(Debug)]
enum Token {
    Source(DataSource),
    Ui(UiMode),
    FrameMode,
}

/// API窓口：環境から生の引数を集めてコアロジックへ
pub fn parse_args() -> Result<AppConfig, DigimaticError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    parse_from_tokens(args)
}

fn parse_from_tokens(args: Vec<String>) -> Result<AppConfig, DigimaticError> {
    // 受信FrameがStr/Binかの指定も追加。なければStr. なので引数は2つか3つとする。
    if args.len() < 2 || args.len() > 3 {
        return Err(invalid_usage());
    }

    let mut source = None;
    let mut ui = None;
    let mut is_bin = false; // FrameFormat のモード判別用

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
            Token::FrameMode => {
                is_bin = true;
            }
        }
    }

    // シャドーイング 中身を確定させる -> unwrap() 対策
    // これをしないでOk(AppConfig) を組み立てようとしても uiがOptionのままでConsole_modeが確定できない
    let source = source.ok_or(invalid_usage())?;
    let ui = ui.ok_or(invalid_usage())?;

    // config.rs の builder 呼び出し
    Ok(crate::config::AppConfig::build(source, ui, is_bin))
}

fn normalize_arg(arg: &str) -> Result<Token, DigimaticError> {
    let normalized = arg.to_lowercase();
    //let normalized = normalized.trim_start_matches('-');

    //TODO 一般的なハイフンありの形にもっていく
    match normalized.as_str() {
        "--sim" | "-s" => Ok(Token::Source(DataSource::Sim)),
        "--actual" | "-a" => Ok(Token::Source(DataSource::Actual)),
        "--gui" | "-g" => Ok(Token::Ui(UiMode::Gui)),
        "--cli" | "-c" => Ok(Token::Ui(UiMode::Cli)),
        "--bin" | "-b" => Ok(Token::FrameMode),
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
        assert!(matches!(
            normalize_arg("--sim").unwrap(),
            Token::Source(DataSource::Sim)
        ));
        assert!(matches!(
            normalize_arg("-s").unwrap(),
            Token::Source(DataSource::Sim)
        ));
    }

    #[test]
    fn test_normalize_arg_ui() {
        assert!(matches!(
            normalize_arg("--gui").unwrap(),
            Token::Ui(UiMode::Gui)
        ));
        assert!(matches!(
            normalize_arg("-g").unwrap(),
            Token::Ui(UiMode::Gui)
        ));
    }

    #[test]
    fn test_duplicate_detection() {
        // ✨ 正しいハイフン付きに変更：sourceの重複を正しく検知できるか
        let args_dup_source = vec!["--sim".to_string(), "--actual".to_string()];
        assert!(parse_from_tokens(args_dup_source).is_err());

        // ✨ 正しいハイフン付きに変更：uiの重複を正しく検知できるか
        let args_dup_ui = vec!["--gui".to_string(), "--cli".to_string()];
        assert!(parse_from_tokens(args_dup_ui).is_err());
    }

    #[test]
    fn test_parse_success_with_binary_option() {
        let args = vec![
            "--cli".to_string(),
            "--actual".to_string(),
            "--bin".to_string(),
        ];
        let config = parse_from_tokens(args).unwrap();

        assert_eq!(config.source, DataSource::Actual);
        assert_eq!(config.ui, UiMode::Cli);
        assert!(matches!(config.format, crate::config::FrameFormat::Bin));
    }

    #[test]
    fn test_strict_hyphen_safety_guards() {
        // ハイフンが足りない（生文字列）場合は「不正な引数」として弾くこと
        assert!(normalize_arg("sim").is_err());
        assert!(normalize_arg("gui").is_err());
        assert!(normalize_arg("g").is_err());

        // ハイフンが多すぎる（タイポなど）場合も確実に弾くこと
        assert!(normalize_arg("---sim").is_err());
        assert!(normalize_arg("--").is_err());
        assert!(normalize_arg("-").is_err());
    }

    #[test]
    fn test_case_insensitivity_with_hyphens() {
        assert!(matches!(
            normalize_arg("--SiM").unwrap(),
            Token::Source(DataSource::Sim)
        ));
        assert!(matches!(
            normalize_arg("--GUI").unwrap(),
            Token::Ui(UiMode::Gui)
        ));
        assert!(matches!(normalize_arg("-B").unwrap(), Token::FrameMode));
    }

    #[test]
    fn test_parse_from_tokens_strict_combinations() {
        // 正常系：ハイフン付きの正しい組み合わせ
        let valid_args = vec!["--gui".to_string(), "--sim".to_string()];
        let config = parse_from_tokens(valid_args).unwrap();
        assert_eq!(config.source, DataSource::Sim);
        assert_eq!(config.ui, UiMode::Gui);

        // 異常系：1つでもハイフンが抜けていたら全体としてエラーにすること
        let invalid_args = vec!["gui".to_string(), "--sim".to_string()];
        assert!(parse_from_tokens(invalid_args).is_err());
    }
}
