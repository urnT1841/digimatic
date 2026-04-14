
import i18n
import config

def splash_welcome():
    splash_text = """
    === digimatic data receiver with XIAO RP2040 ===
    """ 

    print(splash_text)
    print(f"\n{i18n.t('welcome')}")
    print(f"Language: {config.get('lang')}")
