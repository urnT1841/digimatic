
import machchine
import pin_register


# utility
def get_reg_val(base, offset):
    return machine.mem32[base + offset]

def check_bit(val, shift):
    return (val >> shift) & 1