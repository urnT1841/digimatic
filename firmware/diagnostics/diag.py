import machine
import utils
import pin_register as pr



def pins_state():
    print("=== XIAO RP2040 Pin Status ===")
    # 32bitの塊を1回だけ取得
    all_bits = utils.get_raw_gpio_in()

    # MAP (D0-D10) に登録されている順に表示
    for label, pos in pr.MAP.items():
        # 塊から特定のビット(pos)を抽出
        val = (all_bits >> pos) & 1
        
        status = "HIGH" if val else "LOW "
        print(f"{label:4s} (GPIO{pos:02d}): {status}")

    return 0


def pin_setting_menu():
    print("\n--- Pin Configuration ---")
    label = input("Target Pin (e.g., D10) > ").strip().upper()
    
    if label not in pr.MAP:
        print("Invalid Pin Label.")
        return

    gpio_num = pr.MAP[label]
    
    # ReqPinへのガード処理
    # REQピン (D8 / GPIO 2) への Pull-up 設定をブロック
    # 3.3Vを入れると ノギスが壊れる懸念がある。Pinは再確認
    if gpio_num == pr.MAP["D8"]: # または直接 pr.PIN_REQ と比較
        print(f"⚠️  GUARD: {label} (REQ) cannot be Pull-up/down via this menu.")
        print("Reason: To prevent unintended request signals to the caliper.")
        return

    print(f"Configuring {label} (GPIO{gpio_num})")
    print(" 1: INPUT (None)")
    print(" 2: INPUT (Pull-Up)")
    print(" 3: INPUT (Pull-Down)")
    mode = input("Select Mode > ")

    try:
        if mode == "1":
            machine.Pin(gpio_num, machine.Pin.IN, None)
            print(f"DONE: {label} set to INPUT")
        elif mode == "2":
            machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_UP)
            print(f"DONE: {label} set to PULL_UP")
        elif mode == "3":
            machine.Pin(gpio_num, machine.Pin.IN, machine.Pin.PULL_DOWN)
            print(f"DONE: {label} set to PULL_DOWN")
    except Exception as e:
        print(f"Error: {e}")


def exit_diag():
    print("Exiting...")
    # returnで呼び出し元へ戻す
    return

# メニュー構成を定義
MENU_OPTIONS = {
    "1": ("Pin状態確認", pins_state),
    "2": ("Pin 設定", pin_setting_menu), # 次に作る関数
    "99": ("終了", exit_diag)
}

def show_menu():
    print("\n--- RP2040 DIAG MENU ---")
    for key, (label, _) in MENU_OPTIONS.items():
        print(f" {key:>2}: {label}")


def main_loop():
    while True:
        show_menu()
        sel = input("Select > ")

        if sel in MENU_OPTIONS:
            # 辞書から関数を取り出して実行！
            MENU_OPTIONS[sel][1]()
        else:
            print("Invalid.")


# テスト実行用
if __name__ == "__main__":
    main_loop()

