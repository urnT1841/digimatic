
import random
#import machine
import time

#tx_pin = machine.Pin(27, machine.Pin.OUT)
#rx_pin = machine.Pin(28, machine.Pin.IN)


def make_nibble_list(frame):
    
    # 受け取ったリストの要素をnibble(lsb)にする
    # マイナスで初期化しておいて不備があったらはじかれるように
    digimatic_frame_nibble = [-1] *52
    
    for element in frame:
        e_nibble = []
        e_nibble = make_nibble_bits(element,"lsb")
        digimatic_frame_nibble += e_nibble 
        
    print("print")
    print(f"変換前測定値: {frame}")   
    print(f"変換後: {digimatic_frame_nibble}")
    
    return digimatic_frame_nibble


def sim_measure():
    cd = random.randint(1, 15000) / 100
    return f"{cd:06.2f}"


def build_frame(val_str):
    """
    build mitutoyo digimatic frame
    
    引数:測定値
    返値:digimaticFrame (string)
    
    ### Data Format
    - 4bit = 1digits(d)  d1 ~ d13 までの 13デジット
    - 各デジットは最下位ビット(LSB) から 最上位ビット(MSB) の順に出力します。

    d1: All F (1111)
    d2: All F (1111)
    d3: All F (1111)
    d4: All F (1111)
    d5: sign +:0(0000) , -:8(1000)
    d6: mes data (xxxx)
    d7: mes data (xxxx)
    d8: mes data (xxxx)
    d9  mes dat (xxxx)
    d10: mes data (xxxx)
    d11: mes data (xxxx)
    d12: 小数点位置(1~5) (※1)
    d13 unit 0:mm, 1:inch

    ※1例：0 -> 000000.
           1 -> 00000.0
           5 -> 0.00000    
    """
    
    # list宣言と固定値代入
    digi_frame = [0] *13    # 要素は13個
    digi_frame[0] = "F"  # header
    digi_frame[1] = "F"  # header
    digi_frame[2] = "F"  # header
    digi_frame[3] = "F"  # header
    digi_frame[11] = "2"  # digit point 2 fix
    digi_frame[12] = "0"  # unit: 0 Fix
    
    # sign check
    # 正負のチェック用なので誤差は無視というか,関係ない
    val = float(val_str)
    if val < 0 :
        sign = 8
    else:
        sign = 0
    
    digi_frame[4] = sign  # 0:+, 8:-
    
    
    # 測定値を代入
    # 足りない桁がないように0梅で出しているので頭から回していって小数点はスキップ
    
    idx = 5
    for s in val_str:
        if s != ".":
            digi_frame[idx] = s
            idx += 1
    
    print(f"{digi_frame}")
    return digi_frame




def make_nibble_bits(s, order="lsb"):
    nibble_bits = []

    for digit in s:
        nib = int(digit) & 0x0F

        if order == "lsb":
            bit_range = range(4)              # 0 → 3
        elif order == "msb":
            bit_range = reversed(range(4))    # 3 → 0
        else:
            raise ValueError("order must be 'lsb' or 'msb'")

        for i in bit_range:
            nibble_bits.append((nib >> i) & 1)

    return nibble_bits


def send_binary_bits (send_list):
    for c in send_list:
        tx_pin.value(c)
        time.sleep_us(150)
        print(f"set tx {c}: receive {rx_pin.value()}")



def main():
    
    # 測定データSimデータ生成
    cd = sim_measure()
    # ASCIIのデジマチックフレーム(リスト)生成
    frame_str = build_frame(cd)
    # バイナリフレーム(リスト)を生成
    # Nibbleはlsbで埋込み
    digi_frame = make_nibble_list(frame_str)

    print(f"出膂力するリスト: {digi_frame}")
    #send_binary_bits(digi_frame)
    



if __name__ == '__main__':
    main()
    