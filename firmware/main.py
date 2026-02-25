
import machine
import time

import pin_difinitions
import model_caliper
from led_switch import led_switch
import validation_ruse

# led on/off value
ON = 0
OFF = 1


def send_binary_bits(send_list):
    tx_pin = machine.Pin(27, machine.Pin.OUT)
    clk_pin = machine.Pin(19, machine.Pin.OUT, value=1)

    for c in send_list:
        tx_pin.value(c)
        time.sleep_us(10) # 安定を待つ
        
        # Clock Low (start)
        clk_pin.value(0)
        
        # LED制御やウェイト
        led_switch(g=ON)
        time.sleep_ms(300)
        
        # Clock High
        clk_pin.value(1)
        led_switch(OFF,OFF,OFF)
        time.sleep_ms(300)


def receive_digimatic_frame(rx_Pin):
    bits = []
    
    # 通信開始まで待機（アイドル状態の後の最初の変化を待つ）
    # busy wait なのでよくない。 state machineを導入することを検討
    # ※タイムアウト処理を入れるのが理想的
    
    while len(bits) < 52:
        # CLOCKがLowになる（立ち下がり）のを待つ
        while clk_pin.value() == 1:
            pass
        
        # Lowになった瞬間にDATAを読み取る
        bits.append(rx_pin.value())
        
        # LEDをパルス伸長的に光らせる準備（最初のビットで点灯など）
        if len(bits) == 1:
            led_switch(ON, OFF, OFF)

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while clk_pin.value() == 0:
            pass
            
    # 受信完了後に少し光らせてから消す
    time.sleep_ms(80)
    led_switch(OFF, OFF, OFF)
    
    return bits


def test_send_and_receive(send_list):
    rx_pin, *_, tx_pin = pin_difinitions.init_hardware()
    captured_bits = []
    
    print("--- 送受信テスト開始 ---")
    for i, bit in enumerate(send_list):
        # 1. 送信
        tx_pin.value(bit)
        
        # 2. 受信 (送信した直後のピンの状態を読み取る)
        # 物理的に tx_pin と rx_pin がつながっていれば、bit と同じ値が読めるはず
        read_val = rx_pin.value()
        captured_bits.append(read_val)
        
        # デバッグ表示（4ビットごとに区切ると見やすい）
        sep = " | " if (i + 1) % 4 == 0 else " "
        print(f"{read_val}", end=sep)
        if (i + 1) % 4 == 0 and (i + 1) % 16 == 0: print() # 16bitごとに改行

    print("\n--- 受信完了 ---")
    return captured_bits


def main():
    
    data, clk, req, _, = pin_difinitions.init_hardware()
    led_switch(OFF, OFF, OFF)    # (r, g, b)

    led_switch(OFF, OFF, OFF)

    
def main_sim():

     data, clk, req, _, = pin_difinitions.init_hardware()
     led_switch(OFF, OFF, OFF)    # (r, g, b)

     cd = model_caliper.sim_measure()
     frame_str = model_caliper.build_frame(cd)
     send_list = model_caliper.make_nibble_list(frame_str)
     
     # 送受信シミュレーション
     captured = test_send_and_receive(send_list,data)
     
     print(f"send   : {send_list}")
     print(f"receive: {captured}")
     
     # デコード関数を呼び出す not yet
     # result = decode_digimatic_frame(captured)
     # print(f"デコード結果: {result}")


     led_switch(OFF, OFF, OFF)    # (r, g, b)





if __name__ == '__main__':
    main()
    
