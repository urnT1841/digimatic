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
use crate::sim::sim_config::GenMode;

#[derive(Debug)]
enum Token {
    Source(DataSource),
    Ui(UiMode),
    FrameMode,
    SimMode,
}

/// API窓口：環境から生の引数を集めてコアロジックへ
pub fn parse_args() -> Result<AppConfig, DigimaticError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    parse_from_tokens(args)
}

fn parse_from_tokens(args: Vec<String>) -> Result<AppConfig, DigimaticError> {
    // 最低2つは必要 gui or cli, actuar or sim
    if args.len() < 2 {
        return Err(invalid_usage());
    }

    let mut source = None;
    let mut ui = None;
    let mut is_bin = false;
    let mut sim_mode: Option<GenMode> = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
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
            // 文字列のパース段階で mode 判定された場合
            Token::SimMode => {
                let mode_str = arg.to_lowercase();
                match mode_str.as_str() {
                    "--fixed" => {
                        // 次の引数から確実に数値文字列（123.45 など）を取得
                        let val_str = iter.next().ok_or_else(|| {
                            DigimaticError::Argument(ArgumentError::InvalidArgs(
                                "値が指定されていません".into(),
                            ))
                        })?;
                        // 数値への変換。失敗したら不正な引数エラーへ綺麗に落とす
                        let v: f64 = val_str.parse().map_err(|_| {
                            DigimaticError::Argument(ArgumentError::InvalidArgs(
                                "不正な数値です".into(),
                            ))
                        })?;
                        sim_mode = Some(GenMode::Fixed(v));
                    }
                    "--sin" => {
                        sim_mode = Some(GenMode::SinWave {
                            center: 75.0,
                            amplitude: 50.0,
                            frequency: 0.05,
                            delta: 0.0,
                        });
                    }
                    "--seed" => {
                        let seed_str = iter.next().ok_or_else(|| {
                            DigimaticError::Argument(ArgumentError::InvalidArgs(
                                "シード値が指定されていません".into(),
                            ))
                        })?;
                        let s: u64 = seed_str.parse().map_err(|_| {
                            DigimaticError::Argument(ArgumentError::InvalidArgs(
                                "不正なシード値（整数）です".into(),
                            ))
                        })?;
                        sim_mode = Some(GenMode::Seed(s)); // 🌟 抽出したシード値を enum に包む！
                    }
                    _ => {}
                }
            }
        }
    }

    let source = source.ok_or(invalid_usage())?;
    let ui = ui.ok_or(invalid_usage())?;

    // デフォルトは Random
    let final_sim_mode = sim_mode.unwrap_or(GenMode::Random);

    // build に final_sim_mode を渡す
    Ok(crate::config::AppConfig::build(
        source,
        ui,
        is_bin,
        final_sim_mode,
    ))
}

fn normalize_arg(arg: &str) -> Result<Token, DigimaticError> {
    let normalized = arg.to_lowercase();

    match normalized.as_str() {
        "--sim" | "-s" => Ok(Token::Source(DataSource::Sim)),
        "--actual" | "-a" => Ok(Token::Source(DataSource::Actual)),
        "--gui" | "-g" => Ok(Token::Ui(UiMode::Gui)),
        "--cli" | "-c" => Ok(Token::Ui(UiMode::Cli)),
        "--bin" | "-b" => Ok(Token::FrameMode),
        "--fixed" | "--sin" | "--gaussian" | "--seed" => Ok(Token::SimMode),
        _ => Err(DigimaticError::Argument(ArgumentError::InvalidArgs(
            format!("不正な引数です: {arg}"),
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
