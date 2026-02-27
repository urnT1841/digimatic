
import time
import sys
import select
import gc

from pin_difinitions import ON,OFF
import pin_difinitions as pins
import model_caliper
import led_switch as led
from led_switch import LED_ON, LED_OFF
import validation_rule


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


def receive_digimatic_frame(rx_Pin):
    bits = []
    
    # 通信開始まで待機（アイドル状態の後の最初の変化を待つ）
    # busy wait なのでよくない。 state machineを導入することを検討
    # ※タイムアウト処理を入れるのが理想的
    
    while len(bits) < BIN_FRAME_LENGTH:
        # CLOCKがLowになる（立ち下がり）のを待つ
        while clk.value() == 1:
            pass
        
        # Lowになった瞬間にDATAを読み取る
        bits.append(rx_data.value())
        
        # LEDをパルス伸長的に光らせる準備（最初のビットで点灯など）
        if len(bits) == 1:
            led(LED_ON, LED_OFF, LED_OFF)

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while clk.value() == 0:
            pass
            
    # 受信完了後に少し光らせてから消す
    time.sleep_ms(80)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return bits



def main():

    try:
        current_state = STATE_IDLE
        while True:
            match current_state:
                case STATE_IDLE:
                    # 待機処理
                    if trigger:
                        current_state = STATE_REQUEST
        
                case STATE_REQUEST:
                    # 要求処理
                    current_state = STATE_RECEIVE
        
                case STATE_RECEIVE:
                    # 受信処理
                    # タイムアウト処理を忘れずに
                    current_state = STATE_VALIDATE
        
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

    # image  まだ実装してない
    # send_signal(request)

    return STATE_RECEIVE


def check_stop_command_from_pc():
    """ serialを監視  """

    # PCから STOP 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        line = sys.stdin.readline().strip()
        if line == "STOP":
            return True
    return False


if __name__ == '__main__':
    main()
    
