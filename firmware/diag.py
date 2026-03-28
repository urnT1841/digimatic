import machine
import time
import sys
import select
import pin_definitions as pdef

def pins_state():
    print("=== XIAO RP2040 Pin Status ===")
    # 32bitの塊を1回だけ取得
    all_bits = machine.mem32[pdef.SIO_BASE + pdef.GPIO_IN_OFFSET]
    all_oe   = machine.mem32[pdef.SIO_BASE + pdef.GPIO_OE_OFFSET]

    for label, pos in pdef.GPIO_MAP.items():
        val = (all_bits >> pos) & 1
        dir_str = "OUT" if ((all_oe >> pos) & 1) else "IN"
        status = "HIGH" if val else "LOW "
        print(f"{label:4s} (GPIO:{pos:02d}): {status} [{dir_str}]  [{val}]")


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
    print(" 4: OUTPUT (ON 3.3V)")
    print(" 5: OUTPUT (OFF 0V)")
    print(" 6: Moving (repeat On/Off)")
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
            machine.Pin(gpio_num, machine.Pin.OUT, value=ON)
            print(f"DONE: {label} set to OUTPUT ON (3.3V)")
        elif mode == "5":
            machine.Pin(gpio_num, machine.Pin.OUT, value=OFF)
            print(f"DONE: {label} set to OUTPUT OFF (0.0V)")
        elif mode == "6":
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
    ("1", "Pin状態確認", pins_state, None),
    ("2", "Pin設定", None, [
        ("1", "入力設定", pin_input, None),
        ("2", "出力設定", pin_output, None),
        ("3", "詳細設定", None,[
            ("1", "Pull設定", pin_pull, None),
            ("2", "Drive設定", pin_drive, None),
            ("9", "戻る", None, None),
            ]),
        ("9", "戻る", None, None),
    ]),
    ("3", "Pin状態を動的に変化", pin_repeat_menu, None),
    ("99", "終了", exit_diag, None),
]

def show_menu():
    print("\n--- RP2040 DIAG MENU ---")
    for key, label, *_ in MENU_OPTIONS:
        print(f" {key:>2}: {label}")


def main_loop(menu, parent=None):
    while True:
        show_menu()
        sel = input("Select > ")

        matched = [item for item in MENU_OPTIONS if item[0] == sel]
        #matched = [(key, func) for key, label, func in MENU_OPTIONS if key == sel]

        if not matched:
            print("Invaild.")
            continue

def show_menu(menu, path):
    if path:
        print(f"\n--- RP2040 DIAG MENU [{path}] ---")
    else:
        print("\n--- RP2040 DIAG MENU ---")

    for key, label, *_ in menu:
        print(f" {key:>2}: {label}")


def main_loop(menu, path=""):
    while True:
        show_menu(menu, path)

        sel = input("Select > ").strip()

        # 該当項目検索
        item = next((m for m in menu if m[0] == sel), None)

        if not item:
            print("Invalid.")
            continue

        key, label, func, submenu = item

        # 戻る（トップでは無効）
        if key == "9":
            return

        # サブメニューへ
        if submenu:
            new_path = f"{path}.{key}" if path else key
            main_loop(submenu, new_path)
            continue

        # 関数実行
        if func:
            func()

        # 終了
        if key == "99":
            pdef.init_hardware()
            return


if __name__ == "__main__":
    main_loop(MENU_OPTIONS)