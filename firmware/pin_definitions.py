import time
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
ON = const(1)   # High
OFF = const(0)  # Low


# 電気的な接続は specification にある electrical_connect_image.png
# LEDは別設定 -> led_switch.py 参照

# システム電源・制御設定
EN_1_2V_PIN = D9      # AP2112 EN: レベルシフタ用1.2V電源有効化
DIR_CONTROL_PIN = D10 # SN74LXC8T245 DIR: A->B(受信)固定用

# デジマチック信号入力 (XIAO側)
RX_DATA_PIN = D1
RX_CLK_PIN = D2
RX_DATA_BTN = D3

# 自己テスト用
TX_DATA_PIN = D0
REQ_SW_PIN = D8

# 要求信号出力 (10kΩ抵抗越し)
REQ_OUT_PIN = D7

#pinオブジェクト生成
en = machine.Pin(EN_1_2V_PIN, machine.Pin.OUT,value=0)          # 生成時は出力なし
dir_p = machine.Pin(DIR_CONTROL_PIN, machine.Pin.OUT, value=0)  # 生成時は出力なし (レベルシフタも動かない
rx_data = machine.Pin(RX_DATA_PIN, machine.Pin.IN)      # data (rx)
tx_data = machine.Pin(TX_DATA_PIN, machine.Pin.OUT, value=0)     # tx 
clk = machine.Pin(RX_CLK_PIN, machine.Pin.IN)           # clk
data_btv = machine.PIN(RX_DATA_BTN, machine.Pin.IN)     # data btn
# req は input (Hi-Z) なのでvalue=0はここでの実質的な意味を持たない
# ただしsignalを送るとき Outputモードに遷移するのであらかじめ 0(Low,OFF) を指定しておく
# このPinに対しては Pull_UPしたり Hi にしたりはNG -> 3.3Vが出力されて最悪ノギスが壊れる
req = machine.Pin(REQ_OUT_PIN, machine.Pin.IN)  # req Hi-Z 設定
req.value(OFF)
req_sw = machine.Pin(REQ_SW_PIN, machine.Pin.IN, machine.Pin.PULL_UP)

def init_hardware():
    """  pinオブジェクトの初期化関数 """
    # LDO電源有効かとパスコンの準電待ち 時間は適当。当然計算もしてない
    en.value(ON)
    time.sleep_ms(2)

    dir_p.value(ON) # 方向制御をA->Bに固定 生成時の待ちで安定済み，すぐに設定
    time.sleep_ms(1)  # 系全体の安定待ち （念のため過ぎるか?)
    

# Pinオブジェクトの解放
def cleanup_hardware():
    # LDOへの電源落とすのとReqは Hi-Zにするのは先にやる
    en.value(OFF)
    req.init(mode=machine.Pin.IN)
    
    # picoのPinをOFFに。レジスタ書き込みで一気におとす
    GPIO_OUT_CLR = 0xd0000018
    machine.mem32[GPIO_OUT_CLR] = 0x3FFFFFFF
    
    
def send_request():
    """
      データリクエスト送信
      Hi-Z → LowでRequstになる
      set to 0V (active low)
    """
    req.init(mode=machine.Pin.OUT)
    req.value(0)


def stop_request():
    """
      データリクエスト停止
      Low → Hi-Z 
      set to Hi-Z
    """
    req.init(mode=machine.Pin.IN)