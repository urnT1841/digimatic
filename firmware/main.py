import time
import gc

import pin_definitions as pins
from state_process import state_map, STATE_IDLE, ERR_NONE, session
from i18n import t

# 諸々の初期化とか定数確保が終わったらいったんGCを走らせる
pins.init_hardware()
gc.collect()


def main():
    """ 
    StateMachineで待ち受け
    各Stateは State_process.pyで定義
    """

    try:
        current_state = STATE_IDLE
        err_state = ERR_NONE
        while True:
            # マッチした状態に遷移
            state_handler = state_map.get(current_state)
            if state_handler:
                current_state, err_state = state_handler()
            else:
                current_state = STATE_IDLE

    except KeyboardInterrupt:
        # Finnalyに飛ばすだけ
        raise SystemExit
        # print("Interrrupt by user (ctlr-c etc)")
    
    finally:
        # 後始末
        pins.cleanup_hardware()
        # print("pico stoped")


def splash_welcome():
    splash_text = """
    === digimatic data receiver with XIAO RP2040 ===
    """ 

    print(splash_text)
    print(f"\n{t('welcome')}")
    print('Enter "DIAG", "SIM" or "LANG EN" to start.')



if __name__ == '__main__':
    splash_welcome()
    main()
    
