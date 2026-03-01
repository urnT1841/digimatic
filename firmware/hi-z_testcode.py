from machine import Pin
import time

# Pin 0 を入力(Hi-Z)として初期化
req_pin = Pin(0, Pin.IN)

print("--- 検証開始: Hi-Z 状態 ---")
print("ノギスのプルアップで 1 (High) になるはずです。")

while True:
    # 0.5秒おきに現在の状態を表示
    print(f"現在のReqピンの状態: {req_pin.value()} (電圧を測ってください)")
    
    # Enterを押すとループを抜けて次のステップ（L出力）へ
    # 注意: input()は入力を待つ間、上のprintを止めます
    user_input = input("Enterで L出力(0V) に切替 / 'q'で終了: ")
    
    if user_input == 'q':
        break
    
    # 強制的に L (出力) にして引き込む
    print("--- L出力 (0V) に切り替えます ---")
    req_pin = Pin(0, Pin.OUT, value=0)
    time.sleep(0.1) # 安定待ち
    print(f"現在のReqピンの状態: {req_pin.value()}")
    
    input("Enterで再び Hi-Z(入力) に戻します...")
    req_pin = Pin(0, Pin.IN)