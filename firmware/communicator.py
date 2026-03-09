import time
import sys
import select


from pin_definitions import rx_data, clk, req, req_sw, tx_data, send_request, stop_request, ON, OFF
from led_switch import led_switch as led
from led_switch import LED_ON, LED_OFF



def get_command_from_pc():
    """ serialを監視  """

    # PCから REQ, STOP などの 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        return sys.stdin.readline().strip()
    return None



last_sw_state = 1  # 1:押されていない , 0: 押下
def phy_sw_request():
    global last_sw_state
    """
      Databボタンやタクトスイッチなど外部からReqを出せるように
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
