
import random
import time

import led_switch as led
from led_switch import LED_ON, LED_OFF


def sim_measure():
    """ ノギスの測定値っぽい文字列を返す """
    cd = random.randint(1, 15000) / 100

    return f"{cd:06.2f}"


def build_frame(val_str):
    """
    build mitutoyo digimatic frame
    
    Return: list
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
    # 足りない桁がないように0埋めで出しているので頭から回していって小数点はスキップ
    idx = 5
    for s in val_str:
        if s != ".":
            digi_frame[idx] = s
            idx += 1
    
    return digi_frame


def make_nibble_list(frame):
    """ 受け取ったリストの要素をnibble(lsb)にする """
    digimatic_frame_nibble = []
    
    for element in frame:
        e_nibble = []
        e_nibble = make_nibble_bits(element,"lsb")
        digimatic_frame_nibble += e_nibble 
    
    return digimatic_frame_nibble


def make_nibble_bits(digit, order="lsb"):
    """ 引数を4bit  nibble にして返す """
    nibble_bits = []

    #Fが入っているときは15 (nibbleで 0x0F 全部1)
    if isinstance(digit, str):
        nib = int(digit, 16)
    else:
        nib = int(digit)

    # lsb 4ビット
    nib &= 0x0F

    if order == "lsb":
        bit_range = range(4)              # 0 → 3
    elif order == "msb":
        bit_range = reversed(range(4))    # 3 → 0
    else:
        raise ValueError("order must be 'lsb' or 'msb'")

    for i in bit_range:
        nibble_bits.append((nib >> i) & 1)

    return nibble_bits


def send_binary_bits(send_list):
    """ for sim  """

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


def caliper_sim():
    data = sim_measure()
    frame_str = build_frame(data)
    nib_list = make_nibble_list(frame_str)
    
    return nib_list
    
    
