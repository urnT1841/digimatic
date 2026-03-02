
import time
import sys
import select
import gc

from pin_difinitions import ON, OFF, send_request, stop_request
import pin_difinitions as pins
import led_switch as led
from led_switch import LED_ON, LED_OFF


# pin設定
rx_data, clk, req, tx_data = pins.init_hardware()
led(LED_OFF, LED_OFF, LED_OFF)    # (r, g, b)

BIN_FRAME_LENGTH = 52   # デジマチックのフレームは 52bit

# StateMachine 状態定義
STATE_IDLE = 0
STATE_REQUEST = 1
STATE_RECEIVE = 2
STATE_VALIDATE = 3


# 初期化とか定数確保が終わったらいったんGCを走らせる
gc.collect()


def receive_digimatic_frame(bits_buffer):
    _clk = clk
    _data = rx_data

    # 通信開始まで待機（アイドル状態の後の最初の変化を待つ）
    # busy wait なのでよくない。 state machineを導入することを検討
    # ※タイムアウト処理を入れるのが理想的
    
    while bit_count in  range(BIN_FRAME_LENGTH):
        # CLOCKがLowになる（立ち下がり）のを待つ
        while clk.value() == 1:
            pass
        
        # Lowになった瞬間にDATAを読み取る
        bits_buffer[bit_count] = _data.value()  # データ受信
        
        if bit_count == 0:
        # request を hi-zに戻す
            stop_request() 

        # LEDをパルス伸長的に光らせる準備（最初のビットで点灯など）
        if len(bit_count) == 1:
            led(LED_ON, LED_OFF, LED_OFF)

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while clk.value() == 0:
            pass
            
    # 受信完了後に少し光らせてから消す
    time.sleep_ms(80)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return True



def main():
    #受信用バッファ
    rx_buffer = [0] * BIN_FRAME_LENGTH

    try:
        current_state = STATE_IDLE
        while True:
            match current_state:
                case STATE_IDLE:
                    # 待機処理
                    # TODO:
                    if trigger:
                        current_state = STATE_REQUEST
        
                case STATE_REQUEST:
                    process_request()
                    current_state = STATE_RECEIVE
        
                case STATE_RECEIVE:
                    if receive_caliper_data(rx_buffer):
                        current_state = STATE_VALIDATE
                    else:
                        print("timeout")
                        current_state = STATE_IDLE

                case STATE_VALIDATE:
                    validate(rx_buffer)
                    current_state = STATE_IDLE
        
                case _:
                    # とりあえず上記以外は待ち
                    current_state = STATE_IDLE

            if check_stop_command_from_pc():
                break


    
    except KeyboardInterrupt:
        print("Interrrupt by user (ctlr-c etc)")
    
    finally:
        # 後始末
        pins.cleanup_hardware()
        print("pico stoped")





def process_idel():
    """  待ち受け    """
    
    # 250ms分 Request と Data の変化を監視
    # 変化があればそのStateを返す，変化なければそのままで返す

    return STATE_IDLE        
        
def process_request():
    """ スイッチ，PCからのトリガを受け caliperにRequestを送る  """
    send_request()



def check_stop_command_from_pc():
    """ serialを監視  """

    # PCから STOP 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        line = sys.stdin.readline().strip()
        if line == "STOP":
            return True
    return False


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
    
