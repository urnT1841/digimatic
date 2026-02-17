

import random
import time

def gen_cd_measurment():
    val = random.randint(1, 15000)
    return val


def to_str_for_frame(val):
    return f"{val:07.2f}"


def build_digi_frame(val_str):
    digi_frame = ["0"] * 13

    # fixed values
    digi_frame[0:4]  = ["F","F","F","F"]    # header
    digi_frame[4]  = "0"    # sign  0(+) fix (For test)
    digi_frame[11] = "2"    # point posision 2 fix
    digi_frame[12] = "0"    # unit 0:mm fix

    # 小数点消してlstで流し込む
    digits_only = val_str.replace(".","")
    digi_frame[5:11] = list(digits_only)

    return digi_frame


def sender(frame):
    print(frame)


def sim_from_caliper():
    val = gen_cd_measurment() / 100
    val_str = to_str_for_frame(val)
    frame = build_digi_frame(val_str)
    #list -> 文字列
    frame_string = "".join(frame)

    # バイナリ ただし文字列からのバイナリなので，ノギス実機からの
    # 出力とは異なるので使うときは注意
    # frame_string_bin = frame_string.encode()

    return frame_string


def main():
    for i in range(1, 100):
        digimatic_frame = sim_from_caliper()
        sender(digimatic_frame)

        # 時間調整  .sleep_ms は ミリ秒での指定
        time.sleep_ms(1500)
                

if __name__ == '__main__':
    main()
