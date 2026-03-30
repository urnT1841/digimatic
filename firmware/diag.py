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


def generic_pin_config(title, options_dict, apply_callback):
    print(f"\n--- {title} ---")
    
    # 操作禁止なピン（REQ等）を select_pin 側で弾く
    label, gpio_num = select_pin(guard_req=True) 
    if label is None:
        return

    for key, desc in options_dict.items():
        print(f" {key}: {desc}")

    sel = input("Select > ").strip()
    
    if sel in options_dict:
        # 追加のモード別ガード
        # 例: PULL_UP (sel="2") かつ REQピン (GPIO 1) の場合は、
        # select_pin を抜けてきてもここで最終ブロック
        if title == "入力設定" and sel == "2" and gpio_num == pdef.REQ_GPIO:
             print(f"❌ SAFETY ERROR: {label} cannot be PULL_UP.")
             return

        try:
            apply_callback(gpio_num, sel)
            print(f"DONE: {label} configured as {options_dict[sel]}")
        except Exception as e:
            print(f"Error: {e}")
    else:
        print("Invalid selection.")

def menu_set_input():
    options = {"1": "INPUT", "2": "Pull-Up", "3": "Pull-Down"}
    def apply(num, sel):
        p = machine.Pin(num, machine.Pin.IN, 
                        machine.Pin.PULL_UP if sel=="2" else 
                        machine.Pin.PULL_DOWN if sel=="3" else None)
    generic_pin_config("入力設定", options, apply)

def menu_set_output():
    options = {"1": "ON (3.3V)", "2": "OFF (0V)"}
    def apply(num, sel):
        machine.Pin(num, machine.Pin.OUT, value=(1 if sel=="1" else 0))
    generic_pin_config("出力設定", options, apply)

def menu_set_pull():
    options = {"1": "Pull-Up", "2": "Pull-Down"}
    def apply(num, sel):
        # 既存のピン状態を維持しつつPullだけ変えるのは難しいので再初期化
        machine.Pin(num, machine.Pin.IN, 
                    machine.Pin.PULL_UP if sel=="1" else machine.Pin.PULL_DOWN)
    generic_pin_config("Pull設定", options, apply)



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
            while select.select([sys.stdin], [], [], 0)[0]:
                sys.stdin.read(1)
    
            print(f"Loop {sec}s (Enter to stop)")

            while True:
                val = 1 - val
                p.value(val)
                v_str = "3.3V" if val else "0V"
                print(f"\r  {label}: {v_str}    ", end="")  # 今のモードを表示
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
        ("1", "入力設定", menu_set_input, None),
        ("2", "出力設定", menu_set_output, None),
        ("3", "詳細設定", None, [
            ("1", "Pull設定", menu_set_pull, None),
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


def main_loop(menu=None, path=""):
    if menu is None:
        menu = MENU_OPTIONS
    
    while True:
        show_menu(menu, path)
        sel = input("Select > ").strip()

        # 9(戻り)処理
        if sel == "9":
            if path:
                return
            else:
                print("Already top level.")
                continue
        
        # 選択要素アンパック
        # 下記のジェネレータ式だとうまくいかなかった。Micropythonの制限?
        # item = next((m for m in menu if m[0] == sel), None)
        
        item = None
        for m in menu:
            if m[0] == sel:
                item = m
                break

        if not item:
            print("Invaild selection")
            continue

        key, label, func, submenu = item

        # サブメニュー対応
        if submenu:
            new_path = f"{path}.{key}" if path else key
            main_loop(submenu, new_path)
            continue
        
        if func:
            print("calling:", func)  # ← 追加
            func()
        if key == "99":
            return



if __name__ == "__main__":
    main_loop(MENU_OPTIONS)
