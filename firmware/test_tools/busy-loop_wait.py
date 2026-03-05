
import time

from pin_difinitions import ON, OFF, send_request, stop_request
import pin_difinitions as pins
from led_switch import led_switch as led
from led_switch import LED_ON, LED_OFF


# pin設定
rx_data, clk, req, tx_data , req_sw = pins.init_hardware()
led(LED_OFF, LED_OFF, LED_OFF)    # (r, g, b)


BIN_FRAME_LENGTH = 52   # デジマチックのフレームは 52bit


def receive_busy(bits_buffer):
    """
    Busy-loop でクロック同期受信するシンプル版
    process_receive()と入れ替えて使う
    - Requestは事前に送信済み、または最初に送る
    - Pythonの処理遅延を極力排除
    - bits_buffer: 長さ BIN_FRAME_LENGTH のリスト
    """
    _clk = clk
    _data = rx_data

    # Requestを出しておく
    send_request()

    # 最初のClock立下り待ち
    while _clk.value() == ON:
        pass
    
    #ここで1Bit目読んだほうが良いか？
    #まずは下記を試して2重に持って行ったりするようならここで読む。
    # bits_buffer[0] = _data.value()

    # ビット受信ループ
    # 1bit目を上で読む場合 ranage(1,BIN~)とすること
    for i in range(BIN_FRAME_LENGTH):
        # CLOCK High 待ち
        while _clk.value() == OFF:
            pass
        # CLOCK Low 待ち
        while _clk.value() == ON:
            pass
        # データ読み取り
        bits_buffer[i] = _data.value()

    # 受信完了後にLEDやデバッグ出力
    led(LED_ON, LED_OFF, LED_OFF)
    time.sleep_ms(50)
    led(LED_OFF, LED_OFF, LED_OFF)

    # 一応リクエストを止める
    stop_request()