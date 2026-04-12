import ujson as json
import i18n

_current_lang = "ja"


CONFIG_FILE = "config.json"

_cfg = {}


def load_config():
    global _cfg
    try:
        with open(CONFIG_FILE) as f:
            _cfg = json.load(f)
    except:
        _cfg = {"lang": "en"}  # デフォルト英語
    return _cfg


def save_config():
    with open("config.tmp", "w") as f:
        json.dump(_cfg, f)
    import os
    os.rename("config.tmp", CONFIG_FILE)


def get(key, default=None):
    return _cfg.get(key, default)


def set(key, value):
    _cfg[key] = value
    save_config()