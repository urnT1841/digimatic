# i18nのとっかかり
# こうやるという方針残しておく

_current_lang = "ja"

_data = {
    "ja": {
        "welcome": "フレーム待機中...\n ' DIAG ', ' SIM ' でそれぞれのモードに入れます。",
        "diag_enter": "-- 診断モード開始 --",
        "lang_switched": "言語を切り替えました: ja",
    },
    "en": {
        "welcome": "Waiting for frames...\n Enter 'DIAG' or 'SIM' to switch to the corresponding mode.",
        "diag_enter": "-- Enter Diagnostic Mode --",
        "lang_switched": "Language switched to: en",
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