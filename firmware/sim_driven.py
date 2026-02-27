
import time

import pin_difinitions as pins
import model_caliper
import led_switch as led
from led_switch import LED_ON, LED_OFF


# pin設定
rx_data, clk, req, tx_data = pins.init_hardware()
led(LED_OFF, LED_OFF, LED_OFF)    # (r, g, b)



def main():
    """
    caliper sim model
    """

    led(LED_OFF, LED_OFF, LED_OFF)    # (r, g, b)

    # 各桁をlsbにした52bitのframe
    digi_frame = model_caliper.caliper_sim()

    # 送受信シミュレーション
    led(g=LED_ON)
    captured = test_send_and_receive(digi_frame)
    time.sleep_ms(40)
    led(g=LED_OFF)  # 送受信の前後でLEDを光らせておく
     
    print(f"send   : {digi_frame}")
    print(f"receive: {captured}")
     
    # デコード関数を呼び出す not yet
    # result = decode_digimatic_frame(captured)
    # print(f"デコード結果: {result}")

    led(LED_OFF, LED_OFF, LED_OFF)    # (r, g, b)



def test_send_and_receive(send_list):
    captured_bits = []
    
    print("--- 送受信テスト開始 ---")
    for i, bit in enumerate(send_list):
        # 送信
        tx_data.value(bit)
        time.sleep_us(20)
        
        # 受信 (送信した直後のピンの状態を読み取る)
        # 物理的に tx_pin と rx_pin がつながっていれば、bit と同じ値が読めるはず
        read_val = rx_data.value()
        captured_bits.append(read_val)
        
        # デバッグ表示（4ビットごとに区切ると見やすい）
        sep = " | " if (i + 1) % 4 == 0 else " "
        print(f"{read_val}", end=sep)
        if (i + 1) % 4 == 0 and (i + 1) % 16 == 0: print() # 16bitごとに改行

    print("\n--- 受信完了 ---")
    return captured_bits




if __name__ == '__main__':
    main()