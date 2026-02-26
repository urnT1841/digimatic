
import machine
import time
import gc

from pin_difinitions import ON,OFF
import pin_difinitions as pins
import model_caliper
import led_switch as led
import validation_rule


def send_binary_bits(send_list):
    tx_pin = machine.Pin(pins.D1, machine.Pin.OUT)
    clk_pin = machine.Pin(pins.D6, machine.Pin.OUT, value=1)

    for c in send_list:
        tx_pin.value(c)
        time.sleep_us(10) # 安定を待つ
        
        # Clock Low (start)
        clk_pin.value(0)
        
        # LED制御やウェイト
        led(g=ON)
        time.sleep_ms(300)
        
        # Clock High
        clk_pin.value(1)
        led(OFF,OFF,OFF)
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
            led(ON, OFF, OFF)

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while clk_pin.value() == 0:
            pass
            
    # 受信完了後に少し光らせてから消す
    time.sleep_ms(80)
    led(OFF, OFF, OFF)
    
    return bits




def main():
    
    data, clk, req, _, = pins.init_hardware()
    led(OFF, OFF, OFF)    # (r, g, b)

    gc.collect()


    led(OFF, OFF, OFF)

    







if __name__ == '__main__':
    main()
    
