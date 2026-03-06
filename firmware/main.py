
import time
import gc

import pin_definitions as pins
from validation_rule import STATE_IDLE, STATE_ERROR, STATE_RECEIVE, STATE_REQUEST, STATE_VALIDATE
from validation_rule import ERR_DECODE, ERR_NONE, ERR_READ, ERR_TIMEOUT
from state_process import *
from decoder import BIN_FRAME_LENGTH
from communicator import get_command_from_pc, phy_sw_request


# 諸々の初期化とか定数確保が終わったらいったんGCを走らせる
pins.init_hardware()
current_state = STATE_IDLE
err_state = ERR_NONE
gc.collect()


def main():
    #受信用バッファ
    rx_buffer = [0] * BIN_FRAME_LENGTH

    time.sleep(3)
    print("DEBUG: wait 3s")

    try:
        current_state = STATE_IDLE
        err_state = ERR_NONE
        while True:
            if current_state == STATE_IDLE:
                print("#DEBUG: start waiting")
                current_state , err_state = process_idle()

            elif current_state == STATE_REQUEST:
                print("#DEBUG: start request")
                current_state , err_state = process_request()
        
            elif current_state == STATE_RECEIVE:
                print("#DEBUG: start receive")
                current_state , err_state = process_receive_t(rx_buffer)

            elif current_state == STATE_VALIDATE:
                print("#DEBUG: start validation")
                current_state, err_state = process_validate(rx_buffer)
                
            elif current_state == STATE_ERROR:
                print("#DEBUG: something error")                
                # error messageを送出 err_stateによって分岐
                # TODO: rust側が対応できていないので 現状はpass(何もしない)
                time.sleep(3)
                current_state = STATE_IDLE

            else:
                current_state = STATE_IDLE
            
            # ここにPCからのキー入力受付を入れる
            cmd = get_command_from_pc()
            if cmd == "STOP":
                break
            elif cmd == "REQ":
                current_state = STATE_RECEIVE

            # 外部スイッチからのReq信号生成受付
            if phy_sw_request():
                current_state = STATE_REQUEST

    except KeyboardInterrupt:
        pass
        # print("Interrrupt by user (ctlr-c etc)")
    
    finally:
        # 後始末
        pins.cleanup_hardware()
        # print("pico stoped")


if __name__ == '__main__':
    main()
    
