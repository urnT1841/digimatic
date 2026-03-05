
import time
import sys
import select
import gc

from pin_difinitions import ON, OFF, send_request, stop_request
import pin_difinitions as pins
from led_switch import led_switch as led
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
STATE_ERROR = 4

#エラー定義
ERR_NONE = 0
ERR_TIMEOUT = 1  # 信号が来ない
ERR_READ    = 2  # クロックが途中で途切れた、物理的ノイズ  # TODO: 返す部分は未実装
ERR_DECODE  = 3  # バリデーション（FFFFヘッダ等）失敗

# 初期化とか定数確保が終わったらいったんGCを走らせる
gc.collect()


def main():
    #受信用バッファ
    rx_buffer = [0] * BIN_FRAME_LENGTH

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
                current_state , err_state = process_receive(rx_buffer)

            elif current_state == STATE_VALIDATE:
                print("#DEBUG: start validation")
                # 正規のデコードされた文字列か失敗のNone
                validated = validator(rx_buffer)
                if validated:
                    print("#DEBUG: sucsess validate")
                    send_to_host(validated)
                    current_state = STATE_IDLE
                else:
                    # none (バリデーション失敗)の時
                    current_state = STATE_ERROR

            elif current_state == STATE_ERROR:
                print("#DEBUG: something error")                
                # error messageを送出 err_stateによって分岐
                # TODO: rust側が対応できていないので 現状はpass(何もしない)
                pass

            else:
                # とりあえず上記以外は待ち
                current_state = STATE_IDLE

            if check_stop_command_from_pc():
                break
        
        # ここにPCからのキー入力受付を入れる

    except KeyboardInterrupt:
        pass
        # print("Interrrupt by user (ctlr-c etc)")
    
    finally:
        # 後始末
        pins.cleanup_hardware()
        # print("pico stoped")


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
            print("#DEBUG: clock_low (ideling)")
            time.sleep_us(CLOCK_CHECK_TIME)
            if clk.value() == 0:
                print("#DEBUG: move to receive (clock still low)")
                return STATE_RECEIVE , ERR_NONE

    return STATE_IDLE , ERR_NONE


def process_request():
    """ スイッチ, PCからのトリガを受け caliperにRequestを送る  """
    print("#DEBUG: send request")
    send_request()

    return STATE_RECEIVE , ERR_NONE


def process_receive(bits_buffer):
    print("#DEBUG: start receive")
    # Clock に同期しつつdataを52bit読み込む
    _clk = clk
    _data = rx_data
    TIMEOUT_US = 500000   # 長すぎかな？  タイムアウト用
    start_tick = time.ticks_us()

    # 呼び出されたとき1ビット目の受信中なので
    # ここに移ってきたときの1ビット目は無条件で受け入れる
    bits_buffer[0] = _data.value()
    stop_request() 
    led(LED_ON, LED_OFF, LED_OFF)
    
    for bit_count in  range(1, BIN_FRAME_LENGTH):
        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while _clk.value() == 0:
            if time.ticks_diff(time.ticks_us(), start_tick) > TIMEOUT_US:
                print("#DEBUG: time out -> Low fix")
                return STATE_ERROR , ERR_TIMEOUT  # タイムアウト失敗 エラー返して呼び出し元で Idle へ

        # 次のCLOCKがLowになるのを待つ
        while _clk.value() == 1:
            if time.ticks_diff(time.ticks_us(), start_tick) > TIMEOUT_US:
                print("#DEBUG: time out -> High fix")
                return STATE_ERROR , ERR_TIMEOUT  # タイムアウト失敗 エラー返して呼び出し元で Idle へ
        
        # Low → DATAを読み取る
        print("#DEBUG: add data")
        bits_buffer[bit_count] = _data.value()  # データ受信        

            
    # 受信完了後に少し光らせてから消す
    # この間にデータ来るかも？  clock 監視ループの中に入れるか
    time.sleep_ms(80)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return STATE_VALIDATE , ERR_NONE


def check_stop_command_from_pc():
    """ serialを監視  """

    # PCから STOP 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        line = sys.stdin.readline().strip()
        if line == "STOP":
            return True
    return False


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
    
