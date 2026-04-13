import machine
import time
import sys
import select
import pin_definitions as pdef
from i18n import t


def pins_state():
    print(t("pin_status_title"))

    all_bits = machine.mem32[pdef.SIO_BASE + pdef.GPIO_IN_OFFSET]
    all_oe   = machine.mem32[pdef.SIO_BASE + pdef.GPIO_OE_OFFSET]

    for label, pos in pdef.GPIO_MAP.items():
        val = (all_bits >> pos) & 1
        dir_str = "OUT" if ((all_oe >> pos) & 1) else "IN"
        status = "HIGH" if val else "LOW "
        print(f"{label:4s} (GPIO:{pos:02d}): {status} [{dir_str}] [{val}]")


def select_pin(guard_req=True):
    label = input(t("target_pin")).strip().upper()

    if label not in pdef.GPIO_MAP:
        print(t("invalid_pin"))
        return None, None

    gpio_num = pdef.GPIO_MAP[label]

    if guard_req and pdef.is_protected(gpio_num):
        print(t("guard_error").format(label=label, gpio=gpio_num))
        return None, None

    return label, gpio_num


def generic_pin_config(title, options_dict, apply_callback):
    print(f"\n--- {t("title")} ---")
    
    # 操作禁止なピン（REQ等）を select_pin 側で弾く
    label, gpio_num = select_pin(guard_req=True) 
    if label is None:
        return

    for key, desc in options_dict.items():
        print(f" {key}: {desc}")

    sel = input(t("select")).strip()
    
    if sel in options_dict:
        # 追加のモード別ガード
        # 例: PULL_UP (sel="2") かつ REQピン (GPIO 1) の場合は、
        # select_pin を抜けてきてもここで最終ブロック
        if title == "input_config" and sel == "2" and gpio_num == pdef.REQ_GPIO:
             print(t("safety_pullup_error").format(label=label))
             return

        try:
            apply_callback(gpio_num, sel)
            print(t("done").format(label=label, mode=t(options_dict[sel])))
        except Exception as e:
            print(t("error").format(err=e))
    else:
        print(t("invalid_selection"))


def menu_set_input():
    options = {"1": "INPUT", "2": "Pull-Up", "3": "Pull-Down"}
    def apply(num, sel):
        p = machine.Pin(num, machine.Pin.IN, 
            machine.Pin.PULL_UP if sel=="2" else 
            machine.Pin.PULL_DOWN if sel=="3" else None)
    generic_pin_config("input_config", options, apply)


def menu_set_output():
    options = {"1": "ON (3.3V)", "2": "OFF (0V)"}
    def apply(num, sel):
        machine.Pin(num, machine.Pin.OUT, value=(1 if sel=="1" else 0))
    generic_pin_config("output_config", options, apply)


options = {
    "1": "opt_input",
    "2": "opt_pull_up",
    "3": "opt_pull_down"
}

def menu_set_pull():
    options = {
        "1": "opt_pull_up",
        "2": "opt_pull_down"
    }

    def apply(num, sel):
        machine.Pin(
            num,
            machine.Pin.IN,
            machine.Pin.PULL_UP if sel == "1" else machine.Pin.PULL_DOWN
        )

    generic_pin_config("pull_config", options, apply)


def pin_drive():
    print("\n--- " + t("drive_config") + " ---")
    print(t("not_implemented"))

# 動作系
def pin_repeat(label, gpio_num):
    if pdef.is_protected(gpio_num):
        print(t("protected").format(label=label, gpio=gpio_num))
        return

    print("\n" + t("voltage_test").format(label=label, gpio=gpio_num))
    print(f" {t('enter_toggle')}")
    print(f" {t('auto_loop')}")
    print(f" {t('quit')}")

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
    
            print(t("loop").format(sec=sec))

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
            print(t("stopped"))
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
    print(t("exiting"))


# メニュー構造
MENU_OPTIONS = [
    ("1", "pin_status", pins_state, None),
    ("2", "pin_config", None, [
        ("1", "input_config", menu_set_input, None),
        ("2", "output_config", menu_set_output, None),
        ("3", "detail_config", None, [
            ("1", "pull_config", menu_set_pull, None),
            ("2", "drive_config", pin_drive, None),
            ("9", "back", None, None),
        ]),
        ("9", "back", None, None),
    ]),
    ("3", "dynamic_pin", pin_repeat_menu, None),
    ("99", "exit", exit_diag, None),
]

# メニュー実行
def show_menu(menu, path_key="", path_label=""):
    if path_key:
        print("\n" + t("menu_path").format(path_key=path_key, path_label=path_label))
    else:
        print("\n" + t("menu"))

    for key, label, *_ in menu:
        print(f" {key:>2}: {t(label)}")


def main_loop(menu=None, path_key="", path_label=""):
    if menu is None:
        menu = MENU_OPTIONS
    
    while True:
        show_menu(menu, path_key, path_label)
        sel = input(t("select")).strip()

        # 9(戻り)処理
        if sel == "9":
            if path_key:
                return
            else:
                print(t("already_top"))
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
            print(t("invalid_selection"))
            continue

        key, label, func, submenu = item

        # サブメニュー対応
        if submenu:
            new_path_key = f"{path_key}.{key}" if path_key else key
            new_path_label = f"{path_label}/{t(label)}" if path_label else t(label) # 人間にはこっちのほうが読みやすい
            main_loop(submenu, new_path_key, new_path_label)
            continue
        
        if func:
            print(t("call_func"), func)  # ← 追加
            func()
        if key == "99":
            return



if __name__ == "__main__":
    main_loop(MENU_OPTIONS)
