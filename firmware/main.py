
import time
import sys
import select
import gc

from pin_difinitions import ON, OFF, send_request, stop_request
import pin_difinitions as pins
import led_switch as led
from led_switch import LED_ON, LED_OFF
from decoder import validator

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


def main():
    #受信用バッファ
    rx_buffer = [0] * BIN_FRAME_LENGTH

    try:
        current_state = STATE_IDLE
        while True:
            if current_state == STATE_IDLE:
                current_state = process_idle()

            elif current_state == STATE_REQUEST:
                current_state = process_request()
        
            elif current_state == STATE_RECEIVE:
                current_state = receive_caliper_data(rx_buffer):

            elif current_state == STATE_VALIDATE:
                if validate(rx_buffer):
                    print("Data is right. send to Host PC")
                    send_date_()
                else:
                    print("bad data. Wait for next data")
                    current_state = STATE_IDLE
        
            else:
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





def process_idle():
    """  待ち受け    """    
    # ノギスからデータ送出 を待ち受ける
    # clock立下り T2:90 - 150us
    # bottom half clock T3: 100us - 150us
    # top hal clock     T4: 100us - 250us
    # なので，1回のidolでの確認時間は 100msとしてその間 clockを監視
    # clock の下がりEdgeを検出して100us過ぎたあともう一度clockを確認して
    # Lowのままだったらreceive stateに移行

    STATE_CHECK_TIME = 100  # ms, 1回あたりのIdel時間
    CLOCK_CHECK_TIME = 100  # us, ノイズ影響を避けるためのclock確認時間

    start_tick = time.ticks_ms()
    while time.ticks_diff(time.ticks_ms(), start_tick) < STATE_CHECK_TIME:
        # クロックが 100us(CLOCK_CHECK_TIME) の間Lowが継続していたか確認
        if clk.value() == 0:
            time.sleep_um(CLOCK_CHECK_TIME)
            if clk.value() == 0:
                return STATE_RECEIVE 
            
        # PCからのリクエストボタン押しなどのチェック
        if check_trigger():
            return STATE_REQUEST
            
    return STATE_IDLE


def process_request():
    """ スイッチ, PCからのトリガを受け caliperにRequestを送る  """
    send_request()

    return STATE_RECEIVE


def process_receive(bits_buffer):
    # Clock に同期しつつdataを52bit読み込む
    _clk = clk
    _data = rx_data
    TIMEOUT_US = 500000   # 長すぎかな？  タイムアウト用

    # 呼び出されたとき1ビット目の受信中なので
    # ここに移ってきたときの1ビット目は無条件で受け入れる
    bits_buffer[0] = _data.value()
    stop_request() 
    led(LED_ON, LED_OFF, LED_OFF)
    
    for bit_count in  range(1, BIN_FRAME_LENGTH):
        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while _clk.value() == 0:
            if time.ticks_diff(time.ticks_us(), start_t) > TIMEOUT_US:
                return STATE_IDLE  # タイムアウト失敗 Idle へ

        # 次のCLOCKがLowになるのを待つ
        while _clk.value() == 1:
            if time.ticks_diff(time.ticks_us(), start_t) > TIMEOUT_US:
                return STATE_IDLE  # タイムアウト失敗 Idle へ
        
        # Low → DATAを読み取る
        bits_buffer[bit_count] = _data.value()  # データ受信        

            
    # 受信完了後に少し光らせてから消す
    # この間にデータ来るかも？  clock 監視ループの中に入れるか
    time.sleep_ms(80)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return STATE_VALIDATE


def process_validate(rx_buffer):
    """ バリデーション → デジマチックフレーム埋め """
    # 中身はこれから実装
    validator(rx_buffer)


    # PCへ送る
    print(digi_frame)

    return STATE_IDLE


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
    
