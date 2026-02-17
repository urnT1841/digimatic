

import random


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
    #文字列送出なのでJoinでくっつける
    frame_string = "".join(frame)
    print(frame_string)


def main():
    for i in range(1, 100):
        val = gen_cd_measurment() / 100
        val_str = to_str_for_frame(val)
        digimatic_frame = build_digi_frame(val_str)
        sender( digimatic_frame)
        
        # 生成文字列とフレームに詰めたものの確認
        # frame_string = "".join(digimatic_frame)
        # print(val_str)
        # print(frame_string)
        

if __name__ == '__main__':
    main()
