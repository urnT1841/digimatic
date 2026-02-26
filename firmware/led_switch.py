

import machine

# LED設定
green_led = machine.Pin(16, machine.Pin.OUT)
red_led = machine.Pin(17, machine.Pin.OUT)
blue_led = machine.Pin(25, machine.Pin.OUT)

# XAIO RP2040のUserLEDは負論理
LED_ON = 0
LED_OFF = 1

def led_switch(r = OFF , g = OFF, b = OFF):
    """   xaio に搭載されている3つのLD制御    
    """

    red_led.value(r)
    green_led.value(g)
    blue_led.value(b)
    
