from machine import Pin
import time
import random

def main():
    led = Pin(17, Pin.OUT)  # XIAO-RP2040 の内蔵LED
    while True:
        s = gen_str()
        sender_str(s)
        led.toggle()
        time.sleep(1)


def sender_str(s):
    print(s)


def gen_str():
    cd = random.randint(1, 15000)
    return round(cd / 100 ,2)


main()