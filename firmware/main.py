

import random
import time
from machine import Pin

# 17: Red, 16: Green, 25: Blue (XIAO RP2040の場合)
# 緑以外のLEDは消しておく
#led_green = Pin(16, Pin.OUT).value(1)  # XIAO-RP2040 の内蔵LED
#Pin(17, Pin.OUT).value(1)
#Pin(25, Pin.OUT).value(1)

def gen_cd_measurment():
    """ CDデータ生成 100で割って 整数部 ３桁，小数点以下２桁となる値 """
    val = random.randint(1, 15000)
    return val


def to_str_form_frame(val):
    """頭の不足桁は0で埋めて返す """
    return f"{val:07.2f}"


def build_digi_frame(val_str):
    """ デジマチックのフレームっぽいリストを返す  """
    digi_frame = ["0"] * 13

    # fixed values
    digi_frame[0:4]  = ["F","F","F","F"]    # header
    digi_frame[4]  = "0"    # sign  0(+) fix (For test)
    digi_frame[11] = "2"    # point posision 2 fix
    digi_frame[12] = "0"    # unit 0:mm fix

    # 測定値相当は小数点消してlstで流し込む
    digits_only = val_str.replace(".","")
    digi_frame[5:11] = list(digits_only)

    return digi_frame


def sender(frame):
    led_green.values(0)
    print(frame)
    time.sleep_ms(50)   # 50msだと短いかな？
    led_green.values(1)


def sim_from_caliper():
    val = gen_cd_measurment() / 100
    val_str = to_str_form_frame(val)
    frame = build_digi_frame(val_str)
    #list -> 文字列
    frame_string = "".join(frame)

    # バイナリ ただし文字列からのバイナリなので，ノギス実機からの
    # 出力とは異なる。使うときは注意
    # frame_string_bin = frame_string.encode()

    return frame_string


def main():
    while True:
        digimatic_frame = sim_from_caliper()
        sender(digimatic_frame)

        # 時間調整  .sleep_ms は ミリ秒での指定
        time.sleep_ms(1500)
                

if __name__ == '__main__':
    main()
