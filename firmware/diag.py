import machine
import time
import sys
import select
import pin_definitions as pdef


def get_reg_val(base, offset):
    return machine.mem32[base + offset]


# 現状の全GPIOの状態（32bit）を返す
def get_raw_gpio_in():
    return machine.mem32[pdef.SIO_BASE + pdef.GPIO_IN_OFFSET]


def pins_state():
    print("=== XIAO RP2040 Pin Status ===")
    all_bits = get_raw_gpio_in()

    for label, pos in pdef.GPIO_MAP.items():
        val = (all_bits >> pos) & 1
        status = "HIGH" if val else "LOW "
        print(f"{label:4s} (GPIO{pos:02d}): {status}  [{val}]")
    return 0


def select_pin(guard_req=True):
    label = input("Target Pin (e.g., D10) > ").strip().upper()
    if label not in pdef.GPIO_MAP:
        print("Invalid Pin Label.")
        return None, None

    gpio_num = pdef.GPIO_MAP[label]

    # GUI/設定でのガード
    if guard_req and pdef.is_protected(gpio_num):
        print(f"⚠️  GUARD: {label} (GPIO{gpio_num}) is restricted.")
        return None, None

    return label, gpio_num


def pin_setting_menu():
    print("\n--- Pin Configuration ---")
    label, gpio_num = select_pin(guard_req=True)
    if label is None:
        return

    print(f"Configuring {label} (GPIO{gpio_num})")
    print(" 1: INPUT (None)")
    print(" 2: INPUT (Pull-Up)")
    print(" 3: INPUT (Pull-Down)")
    print(" 4: Moving (repeat On/Off)")
    mode = input("Select Mode > ")

    try:
        # 最終防御: 直接ピンを保護
        if pdef.is_protected(gpio_num):
            print(f"❌ PROTECTED: {label} (GPIO{gpio_num})")
            return

        if mode == "1":
            machine.Pin(gpio_num, machine.Pin.IN)
            print(f"DONE: {label} set to INPUT")
        elif mode == "2":
            machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_UP)
            print(f"DONE: {label} set to PULL_UP")
        elif mode == "3":
            machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_DOWN)
            print(f"DONE: {label} set to PULL_DOWN")
        elif mode == "4":
            print("Pin stat is moving(w/o PU/PD) (repeat on/off)")
            pin_repeat(label, gpio_num)
    except Exception as e:
        print(f"Error: {e}")


def pin_repeat(label, gpio_num):
    # 最終防御
    if pdef.is_protected(gpio_num):
        print(f"❌ PROTECTED: {label} (GPIO{gpio_num})")
        return

    print(f"\n-- Voltage/Logic Test: {label} (GPIO{gpio_num}) --")
    print(" [Enter]: Toggle 3.3V/0V (Manual)")
    print(" [数字] : Auto Loop (sec)")
    print(" [q]    : Back to Menu")
    
    p = machine.Pin(gpio_num, machine.Pin.OUT)
    val = 0
    p.value(val)

    while True:
        v_str = "3.3V" if val else "0V"
        cmd = input(f"({label} Output: {v_str}) > ").strip().lower()

        if cmd == 'q':
            p.init(mode=machine.Pin.IN) 
            break
            
        try:
            sec = float(cmd)
            print(f"Looping every {sec}s... [Press Enter to Stop]")

            while True:
                val = 1 - val
                p.value(val)
                
                start_time = time.ticks_ms()
                interrupted = False
                
                while time.ticks_diff(time.ticks_ms(), start_time) < (sec * 1000):
                    r, _, _ = select.select([sys.stdin], [], [], 0)
                    if r:
                        sys.stdin.readline()
                        interrupted = True
                        break
                    time.sleep_ms(50)
                
                if interrupted:
                    print("Stopped Auto Loop.")
                    p.value(0)
                    break

        except ValueError:
            val = 1 - val
            p.value(val)


def pin_repeat_menu():
    label, gpio_num = select_pin(guard_req=True)
    if label is None:
        return
    pin_repeat(label, gpio_num)


def exit_diag():
    print("Exiting...")
    return


MENU_OPTIONS = [
    ("1",  "Pin状態確認",          pins_state),
    ("2",  "Pin設定",              pin_setting_menu),
    ("3",  "Pin状態を動的に変化",  pin_repeat_menu),
    ("99", "終了",                 exit_diag),
]


def show_menu():
    print("\n--- RP2040 DIAG MENU ---")
    for key, label, _ in MENU_OPTIONS:
        print(f" {key:>2}: {label}")


def main_loop():
    while True:
        show_menu()
        sel = input("Select > ")

        matched = [(key, func) for key, label, func in MENU_OPTIONS if key == sel]

        if matched:
            key, func = matched[0]
            func()
            if sel == "99":
                pdef.init_hardware()
                break
        else:
            print("Invalid.")


if __name__ == "__main__":
    main_loop()