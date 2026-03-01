import machine
from micropython import const

# seeed studio xiao rp2040 pin difinition
D0 = 26
D1 = 27
D2 = 28
D3 = 29
D4 = 6
D5 = 7
D6 = 0
D7 = 1
D8 = 2
D9 = 4
D10 = 3

# 論理定数 正論理 DataPinはこれで制御
# LED Pinに関してはLDEの中で定義。なおLEDは負論理 high = off
ON = const(1)
OFF = const(0)


# 電気的な接続は specification にある electrical_connect_image.png
# LEDは別設定 -> led_switch.py 参照

# システム電源・制御設定
EN_1_2V_PIN = D9     # AP2112 EN: レベルシフタ用1.2V電源有効化
DIR_CONTROL_PIN = D10 # SN74LXC8T245 DIR: A->B(受信)固定用

# デジマチック信号入力 (XIAO側)
RX_DATA_PIN = D1
RX_CLK_PIN = D2

# 自己テスト用
TX_DATA_PIN = D0

# 要求信号出力 (10kΩ抵抗越し)
REQ_OUT_PIN = D7

#pinオブジェクト生成
en = machine.Pin(EN_1_2V_PIN, machine.Pin.OUT)
dir_p = machine.Pin(DIR_CONTROL_PIN, machine.Pin.OUT)
rx_data = machine.Pin(RX_DATA_PIN, machine.Pin.IN)      # data (rx)
clk = machine.Pin(RX_CLK_PIN, machine.Pin.IN)           # clk
req = machine.Pin(REQ_OUT_PIN, machine.Pin.IN)          # req Hi-Z 設定
tx_data = machine.Pin(TX_DATA_PIN, machine.Pin.OUT)     # tx

# OFF (0)を指定しておく
req.value(OFF)

# pinオブジェクトの初期化関数
def init_hardware():
    en.value(ON)   # LDO電源有効化
    dir_p.value(OFF) # 方向制御をA->Bに固定
    
    # INのPinが不安定なら pullup 設定(value = 1)を入れてみる
    return (
        rx_data, clk, req, tx_data
    )

# Pinオブジェクトの解放
def cleanup_hardware():
    # LDOへの電源落とすのとReqは hHi-Zにするのは先にやる
    en.value(OFF)
    req.init(mode=machine.Pin.IN)
    
    # picoのPinをOFFに。レジスタ書き込みで一気に
    GPIO_OUT_CLR = 0xd0000018
    machine.mem32[GPIO_OUT_CLR] = 0x3FFFFFFF
    
    

