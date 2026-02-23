
import random
import machine
import time

# ピン設定

tx_pin = machine.Pin(27, machine.Pin.OUT)
rx_pin = machine.Pin(28, machine.Pin.IN)
clk_pin = machine.Pin(19, machine.Pin.OUT, value=1)

ON = 0
OFF = 1
green_led = machine.Pin(16, machine.Pin.OUT)
red_led = machine.Pin(17, machine.Pin.OUT)


def sim_measure():
    cd = random.randint(1, 15000) / 100
    return f"{cd:06.2f}"


def build_frame(val_str):
    """
    build mitutoyo digimatic frame

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
    # 足りない桁がないように0梅で出しているので頭から回していって小数点はスキップ
    
    idx = 5
    for s in val_str:
        if s != ".":
            digi_frame[idx] = s
            idx += 1
    
    print(f"{digi_frame}")
    return digi_frame


def make_nibble_list(frame):
    
    # 受け取ったリストの要素をnibble(lsb)にする
    # マイナスで初期化しておいて不備があったらはじかれるように
    digimatic_frame_nibble = []
    
    for element in frame:
        e_nibble = []
        e_nibble = make_nibble_bits(element,"lsb")
        digimatic_frame_nibble += e_nibble 
    
    return digimatic_frame_nibble


def make_nibble_bits(digit, order="lsb"):
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
    for c in send_list:
        tx_pin.value(c)
        time.sleep_us(10) # 安定を待つ
        
        # Clock Low (start)
        clk_pin.value(0)
        
        # LED制御やウェイト
        green_led.value(ON)
        time.sleep_ms(300)
        
        # Clock High
        clk_pin.value(1)
        green_led.value(OFF)
        time.sleep_ms(300)

    print(f"set tx {c}: receive {rx_pin.value()}")


def receive_digimatic_frame():
    bits = []
    
    # 通信開始まで待機（アイドル状態の後の最初の変化を待つ）
    # ※タイムアウト処理を入れるのが理想的
    
    while len(bits) < 52:
        # CLOCKがLowになる（立ち下がり）のを待つ
        while clk_pin.value() == 1:
            pass
        
        # Lowになった瞬間にDATAを読み取る
        bits.append(rx_pin.value())
        
        # LEDをパルス伸長的に光らせる準備（最初のビットで点灯など）
        if len(bits) == 1:
            red_led.value(ON)

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while clk_pin.value() == 0:
            pass
            
    # 受信完了後に少し光らせてから消す
    time.sleep_ms(80)
    red_led.value(OFF)
    
    return bits


def test_send_and_receive(send_list):
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
    green_led.value(OFF)
    red_led.value(OFF)
    
    cd = sim_measure()
    frame_str = build_frame(cd)
    send_list = make_nibble_list(frame_str)
    
    # 送受信シミュレーション
    captured = test_send_and_receive(send_list)
    
    print(f"send   : {send_list}")
    print(f"receive: {captured}")
    
    # デコード関数を呼び出す not yet
    # result = decode_digimatic_frame(captured)
    # print(f"デコード結果: {result}")


    green_led.value(OFF)
    red_led.value(OFF)


def main2():
    
    green_led.value(OFF)
    red_led.value(OFF)
    
    # 測定データSimデータ生成
    cd = sim_measure()
    # ASCIIのデジマチックフレーム(リスト)生成
    frame_str = build_frame(cd)
    print(f"frame_str: {frame_str}")
    # バイナリフレーム(リスト)を生成
    # Nibbleはlsbで埋込み
    digi_frame = make_nibble_list(frame_str)

    print(f"cd_sim: {cd}")
    print(f"出力するリスト: {digi_frame}")
    send_binary_bits(digi_frame)
    
    green_led.value(OFF)
    red_led.value(OFF)
 



if __name__ == '__main__':
    main()
    
