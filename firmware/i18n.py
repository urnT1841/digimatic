# i18nのとっかかり
# こうやるという方針残しておく

import ujson as json
import os


_data = {}
_fallback = {}

# フォルダにある言語ファイル一覧
def list_languages():
    langs = []
    for f in os.listdir():
        if f.startswith("lang_") and f.endswith(".json"):
            langs.append(f[5:-5])
    return langs

#言語ファイル読み込み (フォールバックあり)
def load_lang(code):
    global _data, _fallback

    if not _fallback:
        try:
            with open("lang_en.json") as f:
                _fallback = json.load(f)
        except OSError:
            _fallback = {}

        try:
            filename = "lang_{}.json".format(code)
            with open(filename) as f:
                _data = json.load(f)
                return True
        except OSError:
            _data = dict(_fallback) # 読み込めなかったらフォールバックを正とする
            return False


def t(key):
    return _data.get(key, _fallback.get(key, key))


def check_keys():
    missing = []
    for k in _fallback:
        if k not in _data:
            missing.append(k)
    return missing