import machine

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

# 論理定数
ON = 0
OFF = 1

# ピンオブジェクトの初期化関数
def init_hardware():
    # 電源有効化
    en = machine.Pin(EN_1_2V_PIN, machine.Pin.OUT)
    en.value(1) 
    
    # 方向制御をA->Bに固定
    dir_p = machine.Pin(DIR_CONTROL_PIN, machine.Pin.OUT)
    dir_p.value(0) 
    
    # INのPinが不安定なら pullup 設定(value = 1)を入れてみる
    return (
        machine.Pin(RX_DATA_PIN, machine.Pin.IN),    # data (rx)
        machine.Pin(RX_CLK_PIN, machine.Pin.IN),       # clk
        machine.Pin(REQ_OUT_PIN, machine.Pin.OUT, value=1),  # req
        machine.Pin(TX_DATA_PIN, machine.Pin.OUT, value=1) # tx
    )