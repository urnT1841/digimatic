
from micropython import const


### レジスタ情報は rp2040 datasheet の下記の賞を参照
### 2.2 Address Map
### 2.3 Processer subsystem
### 2.19 GPIO

# GPIO pin と pin_no の対応
MAP = {
    "D0": 26, "D1": 27, "D2": 28, "D3": 29,
    "D4": 6,  "D5": 7,  "D6": 0,  "D7": 1,
    "D8": 2,  "D9": 4,  "D10": 3
}


# Base Addresses
SIO_BASE        = const(0xd0000000)
PADS_BANK0_BASE = const(0x4001c000)

# Offsets
GPIO_IN_OFFSET  = const(0x004) # 入力値(H/L)
GPIO_OE_OFFSET  = const(0x024) # 出力イネーブル(方向)

# Bit Shifts
# PADS_BANK0 レジスタ内の各ビット
DRIVE_SHIFT     = const(4) # 2bit分 (4,5)
PUE_BIT_SHIFT   = const(3) # Pull-up Enable
PDE_BIT_SHIFT   = const(2) # Pull-down Enable
SCHMITT_SHIFT   = const(1) # Schmitt Trigger (ノギス信号には重要！)


def get_bit_pos(label):
    return MAP.get(label)