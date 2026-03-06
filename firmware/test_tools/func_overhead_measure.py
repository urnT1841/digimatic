import time
import machine

# 120MHz (デフォルト) または 133MHz で動作していることを前提
print(f"CPU Freq: {machine.freq() / 1_000_000} MHz")

def target_func():
    """計測対象の空関数"""
    pass

@micropython.native
def target_func_native():
    """Nativeデコレータをつけた関数"""
    pass

def measure_overhead():
    # 1. 関数呼び出し自体のオーバーヘッド
    t1 = time.ticks_cpu()
    target_func()
    t2 = time.ticks_cpu()
    
    # 2. Native版の呼び出し
    t3 = time.ticks_cpu()
    target_func_native()
    t4 = time.ticks_cpu()

    # 3. 比較用の「何もしない」コスト（ticks_cpu自体の実行時間）
    t5 = time.ticks_cpu()
    t6 = time.ticks_cpu()

    call_cost = time.ticks_diff(t2, t1)
    native_cost = time.ticks_diff(t4, t3)
    pure_ticks_cost = time.ticks_diff(t6, t5)

    print("-" * 30)
    print(f"Standard Call: {call_cost} cycles ({call_cost/120:.2f} us)")
    print(f"Native Call  : {native_cost} cycles ({native_cost/120:.2f} us)")
    print(f"Raw Ticks Cost: {pure_ticks_cost} cycles")
    print("-" * 30)
    print("※ 1ビット(100us)に対してこの『遷移』が占める割合を計算してください。")

if __name__ == "__main__":
    measure_overhead()