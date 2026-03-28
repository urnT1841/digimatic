import machine
import time
import sys
import select
import pin_definitions as pdef

def pins_state():
    print("=== XIAO RP2040 Pin Status ===")

    all_bits = machine.mem32[pdef.SIO_BASE + pdef.GPIO_IN_OFFSET]
    all_oe   = machine.mem32[pdef.SIO_BASE + pdef.GPIO_OE_OFFSET]

    for label, pos in pdef.GPIO_MAP.items():
        val = (all_bits >> pos) & 1
        dir_str = "OUT" if ((all_oe >> pos) & 1) else "IN"
        status = "HIGH" if val else "LOW "
        print(f"{label:4s} (GPIO:{pos:02d}): {status} [{dir_str}] [{val}]")


def select_pin(guard_req=True):
    label = input("Target Pin (e.g., D10) > ").strip().upper()

    if label not in pdef.GPIO_MAP:
        print("Invalid Pin Label.")
        return None, None

    gpio_num = pdef.GPIO_MAP[label]

    if guard_req and pdef.is_protected(gpio_num):
        print(f"⚠️ GUARD: {label} (GPIO{gpio_num}) is restricted.")
        return None, None

    return label, gpio_num

# input設定系
def pin_input():
    print("\n--- INPUT設定 ---")
    label, gpio_num = select_pin()
    if label is None:
        return

    print(" 1: INPUT")
    print(" 2: Pull-Up")
    print(" 3: Pull-Down")

    mode = input("Select > ").strip()

    if mode == "1":
        machine.Pin(gpio_num, machine.Pin.IN)
    elif mode == "2":
        machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_UP)
    elif mode == "3":
        machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_DOWN)
    else:
        print("Invalid.")
        return

    print(f"DONE: {label} INPUT configured")

# output設定系
def pin_output():
    print("\n--- OUTPUT設定 ---")
    label, gpio_num = select_pin()
    if label is None:
        return

    print(" 1: ON (3.3V)")
    print(" 2: OFF (0V)")

    mode = input("Select > ").strip()

    if mode == "1":
        machine.Pin(gpio_num, machine.Pin.OUT, value=1)
    elif mode == "2":
        machine.Pin(gpio_num, machine.Pin.OUT, value=0)
    else:
        print("Invalid.")
        return

    print(f"DONE: {label} OUTPUT configured")

# Pull up / Pull down
def pin_pull():
    print("\n--- Pull設定 ---")
    label, gpio_num = select_pin()
    if label is None:
        return

    print(" 1: Pull-Up")
    print(" 2: Pull-Down")

    mode = input("Select > ").strip()

    if mode == "1":
        machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_UP)
    elif mode == "2":
        machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_DOWN)
    else:
        print("Invalid.")
        return

    print(f"DONE: {label} pull configured")


def pin_drive():
    print("\n--- Drive設定 ---")
    print("Not implemented yet.")


# 動作系
def pin_repeat(label, gpio_num):
    if pdef.is_protected(gpio_num):
        print(f"❌ PROTECTED: {label} (GPIO{gpio_num})")
        return

    print(f"\n-- Voltage Test: {label} (GPIO{gpio_num}) --")
    print(" [Enter]: Toggle")
    print(" [数字] : Auto Loop (sec)")
    print(" [q]    : Back")

    p = machine.Pin(gpio_num, machine.Pin.OUT)
    val = 0
    p.value(val)

    while True:
        v_str = "3.3V" if val else "0V"
        cmd = input(f"({label}:{v_str}) > ").strip().lower()

        if cmd == 'q':
            p.init(mode=machine.Pin.IN)
            break

        try:
            sec = float(cmd)
            print(f"Loop {sec}s (Enter to stop)")

            while True:
                val = 1 - val
                p.value(val)

                start = time.ticks_ms()

                while time.ticks_diff(time.ticks_ms(), start) < (sec * 1000):
                    r, _, _ = select.select([sys.stdin], [], [], 0)
                    if r:
                        sys.stdin.readline()
                        raise KeyboardInterrupt
                    time.sleep_ms(50)

        except KeyboardInterrupt:
            print("Stopped.")
            p.value(0)

        except:
            val = 1 - val
            p.value(val)


def pin_repeat_menu():
    label, gpio_num = select_pin()
    if label is None:
        return
    pin_repeat(label, gpio_num)


def exit_diag():
    pdef.init_hardware()
    print("Exiting...")


# メニュー構造
MENU_OPTIONS = [
    ("1", "Pin状態確認", pins_state, None),
    ("2", "Pin設定", None, [
        ("1", "入力設定", pin_input, None),
        ("2", "出力設定", pin_output, None),
        ("3", "詳細設定", None, [
            ("1", "Pull設定", pin_pull, None),
            ("2", "Drive設定", pin_drive, None),
            ("9", "戻る", None, None),
        ]),
        ("9", "戻る", None, None),
    ]),
    ("3", "Pin状態を動的に変化", pin_repeat_menu, None),
    ("99", "終了", exit_diag, None),
]

# メニュー実行
def show_menu(menu, path):
    if path:
        print(f"\n--- MENU [{path}] ---")
    else:
        print("\n--- MENU ---")

    for key, label, *_ in menu:
        print(f" {key:>2}: {label}")


def main_loop(menu, path=""):
    while True:
        show_menu(menu, path)

        sel = input("Select > ").strip()

        item = next((m for m in menu if m[0] == sel), None)

        if not item:
            print("Invalid.")
            continue

        key, label, func, submenu = item

        if key == "9":
            if path:
                return
            else:
                print("Already top.")
                continue

        if submenu:
            new_path = f"{path}.{key}" if path else key
            main_loop(submenu, new_path)
            continue

        if func:
            func()

        if key == "99":
            pdef.init_hardware()
            return

if __name__ == "__main__":
    main_loop(MENU_OPTIONS)