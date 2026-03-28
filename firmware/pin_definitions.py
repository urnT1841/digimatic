import time
import machine
from micropython import const


### レジスタ情報は rp2040 datasheet の下記の賞を参照
### 2.2 Address Map
### 2.3 Processer subsystem
### 2.19 GPIO
# Base Addresses
SIO_BASE        = const(0xd0000000)
PADS_BANK0_BASE = const(0x4001c000)

# Offsets
GPIO_IN_OFFSET  = const(0x004) # 入力値(H/L)
GPIO_OE_OFFSET  = const(0x020) # 出力イネーブル(方向)

# Bit Shifts
# PADS_BANK0 レジスタ内の各ビット
DRIVE_SHIFT     = const(4) # 2bit分 (4,5)
PUE_BIT_SHIFT   = const(3) # Pull-up Enable
PDE_BIT_SHIFT   = const(2) # Pull-down Enable
SCHMITT_SHIFT   = const(1) # Schmitt Trigger (ノギス信号には重要！)

# 論理定数 正論理 DataPinはこれで制御
# LED Pinに関してはLDEの中で定義。なおLEDは負論理 high = off
ON = const(1)   # High
OFF = const(0)  # Low


# seeed studio xiao rp2040 pin difinition
# GPIO pin と pin_no の対応
GPIO_MAP = {
    "D0": 26, "D1": 27, "D2": 28, "D3": 29,
    "D4": 6,  "D5": 7,  "D6": 0,  "D7": 1,
    "D8": 2,  "D9": 4,  "D10": 3
}

# 使用Pinにラベル付け
# システム電源・制御設定
EN_1_2V_PIN = GPIO_MAP["D9"]      # AP2112 EN: レベルシフタ用1.2V電源有効化
DIR_CONTROL_PIN = GPIO_MAP["D10"] # SN74LXC8T245 DIR: A->B(受信)固定用

# デジマチック信号入力 (XIAO側)
RX_DATA_PIN = GPIO_MAP["D1"]
RX_CLK_PIN = GPIO_MAP["D2"]
RX_DATA_BTN = GPIO_MAP["D3"]

# 自己テスト用
TX_DATA_PIN = GPIO_MAP["D0"]
REQ_SW_PIN = GPIO_MAP["D8"]

# 要求信号出力 (10kΩ抵抗越し)
REQ_OUT_PIN = GPIO_MAP["D7"]
REQ_GPIO = REQ_OUT_PIN
PROTECTED_GPIO = {
    REQ_GPIO
}

def is_protected(gpio_num):
    return gpio_num in PROTECTED_GPIO


PINS = {
    "en": machine.Pin(EN_1_2V_PIN),
    "dir": machine.Pin(DIR_CONTROL_PIN),
    "rx_data": machine.Pin(RX_DATA_PIN),
    "tx_data": machine.Pin(TX_DATA_PIN),
    "clk": machine.Pin(RX_CLK_PIN),
    "data_btn": machine.Pin(RX_DATA_BTN),
    # req は input (Hi-Z) なのでvalue=0はここでの実質的な意味を持たない
    # ただしsignalを送るとき Outputモードに遷移するのであらかじめ 0(Low,OFF) を指定しておく
    # このPinに対しては Pull_UPしたり Hi にしたりはNG -> 3.3Vが出力されて最悪ノギスが壊れる
    "req": machine.Pin(REQ_OUT_PIN),
    "req_sw": machine.Pin(REQ_SW_PIN),
    "unused_D4": machine.Pin(GPIO_MAP["D4"]),
    "unused_D5": machine.Pin(GPIO_MAP["D5"]),
    "unused_D6": machine.Pin(GPIO_MAP["D6"]),
}


# 電気的な接続は specification にある electrical_connect_image.png
# LEDは別設定 -> led_switch.py 参照
# 全GPIOをInput設定 まずは安全側の input+pull-down
def gpio_init():
    for p in PINS.values():
        p.init(machine.Pin.IN, machine.Pin.PULL_DOWN)


def init_hardware():
    """  pinオブジェクトの初期化関数 """
    # diag実施後も init_hardware を呼ぶようにするので，全Pin初期化から。
    gpio_init()

    # 使用Pin初期化
    # LDO電源有効かとパスコンの準電待ち 時間は適当。当然計算もしてない
    PINS["en"].init(mode=machine.Pin.OUT, value=ON)
    time.sleep_ms(2)

    PINS["dir"].init(mode=machine.Pin.OUT, value=ON)
    time.sleep_ms(1)  # 系全体の安定待ち （念のため過ぎるか?)

    PINS["rx_data"].init(mode=machine.Pin.IN)
    PINS["tx_data"].init(mode=machine.Pin.IN)
    PINS["clk"].init(mode=machine.Pin.IN)
    PINS["data_btn"].init(mode=machine.Pin.IN)
    PINS["req"].init(mode=machine.Pin.OUT, value=OFF) #先にOFFで設定してからHi-Zへ移行
    PINS["req"].init(mode=machine.Pin.IN)
    PINS["req_sw"].init(mode=machine.Pin.IN)


def cleanup_hardware():
    # LDOへの電源落とすのとReqは Hi-Zにするのは先にやる
    PINS["en"].value(OFF)
    PINS["req"].init(mode=machine.Pin.IN)
    
    # picoのPinをOFFに。レジスタ書き込みで一気におとす
    GPIO_OUT_CLR = 0xd0000018
    machine.mem32[GPIO_OUT_CLR] = 0x3FFFFFFF
    
    
def send_request():
    """
      データリクエスト送信
      Hi-Z → LowでRequstになる
      set to 0V (active low)
    """
    PINS["req"].init(mode=machine.Pin.OUT)
    PINS["req"].value(0)


def stop_request():
    """
      データリクエスト停止
      Low → Hi-Z 
      set to Hi-Z
    """
    PINS["req"].init(mode=machine.Pin.IN)