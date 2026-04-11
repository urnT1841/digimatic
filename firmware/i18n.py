# i18nのとっかかり
# こうやるという方針残しておく

_current_lang = "ja"

_data = {
    "ja": {
        "welcome": "フレーム待機中...\n ' DIAG ', ' SIM ' でそれぞれのモードに入れます。",
        "diag_enter": "-- 診断モード開始 --",
        "lang_switched": "言語を切り替えました: ja",

        # diag メニュー用
        "menu": "--- MENU ---",
        "pin_status_title": "=== XIAO RP2040 ピン状態 ===",
        "safety_pullup_error": "❌ 安全エラー: {label} は PULL_UP にできません",
        "protected": "❌ 保護対象: {label} (GPIO{gpio})",
        "opt_input": "INPUT",
        "opt_pull_up": "Pull-Up",
        "opt_pull_down": "Pull-Down",
        "menu_path": "--- MENU [{path_key} / {path_label}] ---",
        "select": "Select > ",
        "invalid_selection": "無効な選択です",
        "already_top": "すでにトップレベルです",

        "pin_status": "Pin状態確認",
        "pin_config": "Pin設定",
        "input_config": "入力設定",
        "output_config": "出力設定",
        "detail_config": "詳細設定",
        "pull_config": "Pull設定",
        "drive_config": "Drive設定",
        "back": "戻る",
        "dynamic_pin": "Pin状態を動的に変化",
        "exit": "終了",

        "target_pin": "対象ピン (例: D10) > ",
        "invalid_pin": "無効なピンです",
        "guard_error": "⚠️ GUARD: {label} (GPIO{gpio}) は制限されています",

        "done": "完了: {label} を {mode} に設定しました",
        "error": "エラー: {err}",

        "not_implemented": "未実装です",

        "voltage_test": "-- 電圧テスト: {label} (GPIO{gpio}) --",
        "enter_toggle": "[Enter]: トグル",
        "auto_loop": "[数字]: 自動ループ (秒)",
        "quit": "[q]: 戻る",

        "loop": "ループ {sec}秒 (Enterで停止)",
        "stopped": "停止しました",

        "exiting": "終了中...",
    },
    "en": {
        "welcome": "Waiting for frames...\n Enter 'DIAG' or 'SIM' to switch to the corresponding mode.",
        "diag_enter": "-- Enter Diagnostic Mode --",
        "lang_switched": "Language switched to: en",

        # diag 用
        "menu": "--- MENU ---",
        "pin_status_title": "=== XIAO RP2040 Pin Status ===",
        "safety_pullup_error": "❌ SAFETY ERROR: {label} cannot be set to PULL_UP",
        "protected": "❌ PROTECTED: {label} (GPIO{gpio})",
        "opt_input": "INPUT",
        "opt_pull_up": "Pull-Up",
        "opt_pull_down": "Pull-Down",
        "menu_path": "--- MENU [{path_key} / {path_label}] ---",
        "select": "Select > ",
        "invalid_selection": "Invalid selection",
        "already_top": "Already at top level.",

        "pin_status": "Pin Status",
        "pin_config": "Pin Configuration",
        "input_config": "Input Configuration",
        "output_config": "Output Configuration",
        "detail_config": "Advanced Settings",
        "pull_config": "Pull Configuration",
        "drive_config": "Drive Configuration",
        "back": "Back",
        "dynamic_pin": "Dynamic Pin Control",
        "exit": "Exit",

        "target_pin": "Target Pin (e.g., D10) > ",
        "invalid_pin": "Invalid pin label.",
        "guard_error": "⚠️ GUARD: {label} (GPIO{gpio}) is restricted.",

        "done": "DONE: {label} configured as {mode}",
        "error": "Error: {err}",

        "not_implemented": "Not implemented yet.",

        "voltage_test": "-- Voltage Test: {label} (GPIO{gpio}) --",
        "enter_toggle": "[Enter]: Toggle",
        "auto_loop": "[Number]: Auto Loop (sec)",
        "quit": "[q]: Back",

        "loop": "Loop {sec}s (Press Enter to stop)",
        "stopped": "Stopped.",

        "exiting": "Exiting...",
    }
}

def set_lang(code):
    global _current_lang
    if code in _data:
        _current_lang = code
        return True
    return False


def t(key):
    """翻訳関数 (Translate)"""
    # 辞書にあれば返し、なければキーをそのまま返す（フォールバック）
    return _data[_current_lang].get(key, key)