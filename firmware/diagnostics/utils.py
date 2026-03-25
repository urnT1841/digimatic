import machine
import pin_register

def get_reg_val(base, offset):
    return machine.mem32[base + offset]

# 現状の全GPIOの状態（32bitの塊）をそのまま返す
def get_raw_gpio_in():
    # SIO_BASE + GPIO_IN_OFFSET を見に行く
    return machine.mem32[pin_register.SIO_BASE + pin_register.GPIO_IN_OFFSET]

