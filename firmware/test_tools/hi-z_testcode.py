from machine import Pin
import time

import pin_definitions as pins
from pin_definitions import rx_data, clk, req, req_sw, send_request, stop_request, ON, OFF

pins.init_hardware()


print("--- 検証開始: Hi-Z 状態 ---")
print("ノギスのプルアップで 1 (High) になるはずです。")

clk_buf = []

while True:
    # 0.5秒おきに現在の状態を表示
    print(f"現在のReqピンの状態: {req.value()} (電圧を測ってください)")
    
    # Enterを押すとループを抜けて次のステップ（L出力）へ
    # 注意: input()は入力を待つ間、上のprintを止めます
    user_input = input("Enterで L出力(0V) に切替 / 'q'で終了: ")
    
    if user_input == 'q':
        break
    
    # 強制的に L (出力) にして引き込む
    print("--- L出力 (0V) に切り替えます ---")
    req.init(mode= Pin.IN,value=0)

    # clk_buf = [] などで初期化済みとする
    start_t = time.ticks_us()
    last_val = clk.value()
    limit_us = 100000  # 100ms 監視（500msなら 500000）

    print("監視開始...")

    while time.ticks_diff(time.ticks_us(), start_t) < limit_us:
        curr_val = clk.value()
        # 状態が変わった瞬間だけ記録
        if curr_val != last_val:
            t = time.ticks_diff(time.ticks_us(), start_t)
            clk_buf.append((curr_val, t))
            last_val = curr_val
        
        # メモリ保護：1000回以上の変化があったら念のため止める
        if len(clk_buf) > 1000:
            break

    print(f"監視終了。変化点数: {len(clk_buf)}")
    for val, t in clk_buf:
        print(f"T+{t}us: {'High' if val else 'Low'}")
            
            


    # time.sleep(0.1) # 安定待ち
    # print(f"現在のReqピンの状態: {req_pin.value()}")
    
    input("Enterで再び Hi-Z(入力) に戻します...")
    req.init(mode = Pin.IN)