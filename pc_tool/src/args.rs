//! # 引数解析モジュール
//! args.rs
//!
//! 起動時のコマンドライン引数（`std::env::args`）を
//! 解析し、動作モードを決定します。
//!
//! ## 主な役割
//! - 起動時の引数文字列（例：`--sim`, `--gui` など）のパース
//! - 重複する引数や不整合な組み合わせの排除（バリデーションガード）
//! - 解析結果を安全な `AppConfig` 構造体として組み立て
//!
//! ## 安全性とエラーハンドリング
//! 外部からの不確実な入力（文字列）を最初に受け取る「境界線」となるため、
//! 不正なオプションが指定された場合は、独自定義のエラー型を返して安全に終了させる

use crate::config::{AppConfig, DataSource, UiMode};
use crate::errors::{ArgumentError, DigimaticError};

#[derive(Debug)]
enum Token {
    Source(DataSource),
    Ui(UiMode),
    FrameMode,
}

/// API窓口：環境から生の引数を集めてコアロジックへ流す
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
    let normalized = normalized.trim_start_matches('-');

    match normalized {
        "sim" | "s" => Ok(Token::Source(DataSource::Sim)),
        "actual" | "a" => Ok(Token::Source(DataSource::Actual)),
        "gui" | "g" => Ok(Token::Ui(UiMode::Gui)),
        "cli" | "c" => Ok(Token::Ui(UiMode::Cli)),
        "bin" | "b" => Ok(Token::FrameMode),
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
        // sourceの重複を検知できるか
        let args_dup_source = vec!["sim".to_string(), "actual".to_string()];
        assert!(parse_from_tokens(args_dup_source).is_err());

        // uiの重複を検知できるか
        let args_dup_ui = vec!["gui".to_string(), "cli".to_string()];
        assert!(parse_from_tokens(args_dup_ui).is_err());
    }

    #[test]
    fn test_parse_success_combinations() {
        // 順序が逆（gui sim）でも正しく設定を組み立てられるか検証
        let args = vec!["gui".to_string(), "sim".to_string()];
        let config = parse_from_tokens(args).unwrap();

        assert_eq!(config.source, DataSource::Sim);
        assert_eq!(config.ui, UiMode::Gui);
    }

    #[test]
    fn test_parse_success_with_binary_option() {
        // 🌟 3つ目に `--bin` を指定した3面待ちの組み合わせテスト
        let args = vec!["cli".to_string(), "actual".to_string(), "--bin".to_string()];
        let config = parse_from_tokens(args).unwrap();

        assert_eq!(config.source, DataSource::Actual);
        assert_eq!(config.ui, UiMode::Cli);
        // 🌟 ちゃんと Bin モードが有効になっているか検証！
        assert!(matches!(
            config.format,
            crate::received_data_handler::FrameFormat::Bin
        ));
    }
}
