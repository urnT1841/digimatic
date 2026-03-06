
import time
import sys
import select
import gc

from pin_definitions import rx_data, clk, req, req_sw, tx_data, send_request, stop_request, ON, OFF
import pin_definitions as pins
from led_switch import led_switch as led
from led_switch import LED_ON, LED_OFF
from validation_rule import STATE_IDLE, STATE_ERROR, STATE_RECEIVE, STATE_REQUEST, STATE_VALIDATE
from validation_rule import ERR_DECODE, ERR_NONE, ERR_READ, ERR_TIMEOUT
from state_process import *
from decoder import BIN_FRAME_LENGTH


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
                pass

            else:
                current_state = STATE_IDLE
            
            # ここにPCからのキー入力受付を入れる
            cmd = get_command_from_pc()
            if cmd == "STOP":
                break
            elif cmd == "REQ":
                current_state = STATE_RECEIVE

            # 外部スイッチからのReq信号生成受付
            if phi_sw_request():
                current_state = STATE_REQUEST

    except KeyboardInterrupt:
        pass
        # print("Interrrupt by user (ctlr-c etc)")
    
    finally:
        # 後始末
        pins.cleanup_hardware()
        # print("pico stoped")



def get_command_from_pc():
    """ serialを監視  """

    # PCから REQ, STOP などの 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        return sys.stdin.readline().strip()
    return None



last_sw_state = 1  # 1:押されていない , 0: 押下
def phi_sw_request():
    global last_sw_state
    """
      タクトスイッチなど外部からReqを出せるように
    """
    # pull-upているPinに対してスイッチが押されることでGNDへ落ちる
    # → send_request を送る (mainで State_request になってprocess_requestへ入る)

    current_sw_state = req_sw.value()

    # 離れていて押した最初だけ反応
    sw_pressed = last_sw_state == 1 and current_sw_state == 0
    last_sw_state = current_sw_state

    # チャタリング対策は不要
    #  Trueの場合(押下検知) は Stateが変更されるのでちゃたってもここのロジックを通らない
    #  Falseの場合 Idle に入って100ms戻ってこないのでとうにチャタリングは収まっている
    return sw_pressed



def send_to_host(digi_frame):
    # USB-CDC (print) で Host PC へ
    print(digi_frame)


def send_binary_bits(send_list):

    for c in send_list:
        tx_data.value(c)
        time.sleep_us(10) # 安定を待つ
        
        # Clock Low (start)
        clk.value(OFF)
        
        # LED制御やウェイト
        led(g=LED_ON)
        time.sleep_ms(300)
        
        # Clock High
        clk.value(ON)
        led(LED_OFF,LED_OFF,LED_OFF)
        time.sleep_ms(300)


if __name__ == '__main__':
    main()
    
