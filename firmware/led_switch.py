

import machine
from micropython import const

# LED設定
green_led = machine.Pin(16, machine.Pin.OUT)
red_led = machine.Pin(17, machine.Pin.OUT)
blue_led = machine.Pin(25, machine.Pin.OUT)

# XAIO RP2040のUserLEDは負論理
LED_ON = const(0)
LED_OFF = const(1)

def led_switch(r = LED_OFF , g = LED_OFF, b = LED_OFF):
    """   xaio に搭載されている3つのLD制御    
    """

    red_led.value(r)
    green_led.value(g)
    blue_led.value(b)
    
