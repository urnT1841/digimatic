
import pin_difinitions as pins
import model_caliper
import led_switch as led

ON = pins.ON
OFF = pins.OFF


def main():
    """
    caliper sim model
    """

    data, clk, req, _, = pins.init_hardware()
    led(OFF, OFF, OFF)    # (r, g, b)

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

    led(OFF, OFF, OFF)    # (r, g, b)




def test_send_and_receive(send_list):
    rx_pin, *_, tx_pin = pins.init_hardware()
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




if __name__ == '__main__':
    main()