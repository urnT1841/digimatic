import ujson as json
import i18n

_current_lang = "ja"


CONFIG_FILE = "config.json"

_cfg = {}


def run_interactive_menu():
    print(f"\n--- {i18n.t('config_menu_title')} ---")
    langs = i18n.list_languages()
    for i, l in enumerate(langs):
        print(f" {i}: {l}")
        
    sel = input(i18n.t("select_lang")).strip()
    if sel.isdigit() and int(sel) < len(langs):
        new_lang = langs[int(sel)]
        set("lang", new_lang) # ここで保存される
        i18n.load_lang(new_lang)
        print(i18n.t("lang_switched").format(lang=new_lang))

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


# 便利なメソッド
def get_language():
    return get("lang", "en") # 保存されていなければ英語

# 起動時にこう呼ぶイメージ
# i18n.load_lang(config.get_language())