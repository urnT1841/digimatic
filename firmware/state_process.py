import time

from validation_rule import STATE_IDLE, STATE_ERROR, STATE_RECEIVE, STATE_REQUEST, STATE_VALIDATE
from validation_rule import ERR_DECODE, ERR_NONE, ERR_READ, ERR_TIMEOUT
from pin_definitions import rx_data, clk, req, req_sw, send_request, stop_request, ON, OFF
from led_switch import led_switch as led
from led_switch import LED_ON, LED_OFF
from decoder import BIN_FRAME_LENGTH, validator
from communicator import send_to_host,send_request, stop_request


def process_idle():
    """  待ち受け    """    
    # ノギスからデータ送出 を待ち受ける
    # clock立下り T2:90 - 150us
    # bottom half clock T3: 100us - 150us
    # top hal clock     T4: 100us - 250us
    # なので，1回のidolでの確認時間は 100msとしてその間 clockを監視
    # clock の下がりEdgeを検出して100us過ぎたあともう一度clockを確認して
    # Lowのままだったらreceive stateに移行

    STATE_CHECK_TIME = 100  # ms, 1回あたりのIdel時間
    start_tick = time.ticks_ms()
    prev_clk = clk.value()

    # Req生成
    #debug
    send_request()

    #while time.ticks_diff(time.ticks_ms(), start_tick) < STATE_CHECK_TIME:
    #    now_clk = clk.value()
    #    
    #    # 立ち下がりエッジ検出
    #    if prev_clk == 1 and now_clk == 0:
    #        #print("DEBUG: falling edge detected")
    #        return STATE_RECEIVE , ERR_NONE

    #   prev_clk = now_clk

    return STATE_RECEIVE , ERR_NONE


def process_request():
    """ スイッチ, PCからのトリガを受け caliperにRequestを送る  """
    print("#DEBUG: send request")
    send_request()

    return STATE_RECEIVE , ERR_NONE


def process_receive(bits_buffer):
    print("#DEBUG: start receive logic")
    # Clock に同期しつつdataを52bit読み込む
    _clk = clk
    _data = rx_data
    TIMEOUT_US = 1000   # 長すぎかな？  タイムアウト用

    # Clockのエッジを検出してから飛んでくるのので
    # ここに移ってきたときの1ビット目は無条件で受け入れる
    bits_buffer[0] = _data.value()
    led(LED_ON, LED_OFF, LED_OFF)
    
    for bit_count in  range(1, BIN_FRAME_LENGTH):
        start_tick = time.ticks_us()

        # CLOCKがHighに戻るのを待つ（チャタリング・重複読み防止）
        while _clk.value() == 0:
            if time.ticks_diff(time.ticks_us(), start_tick) > TIMEOUT_US:
                print("#DEBUG: time out -> Low fix")
                return STATE_ERROR , ERR_TIMEOUT  # タイムアウト失敗 エラー返して呼び出し元で Idle へ

        # 次のCLOCKがLowになるのを待つ
        while _clk.value() == 1:
            if time.ticks_diff(time.ticks_us(), start_tick) > TIMEOUT_US:
                print("#DEBUG: time out -> High fix")
                return STATE_ERROR , ERR_TIMEOUT  # タイムアウト失敗 エラー返して呼び出し元で Idle へ


        # 最後のBitを受信し終わってからReqを止める (安定する?)
        stop_request() 

        # Low → DATAを読み取る
        print("#DEBUG: add data : { bit_count} , {_data.value()}")
        bits_buffer[bit_count] = _data.value()  # データ受信        
            
    # 受信完了後に少し光らせてから消す
    # この間にデータ来るかも？  clock 監視ループの中に入れるか
    time.sleep_ms(80)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return STATE_VALIDATE , ERR_NONE


def process_receive_busy2(bits_buffer):
    """
    ミツトヨ・デジマチック 52ビット受信関数 (同期統合版)
    関数遷移のラグを無視するため、1ビット目の立ち下がりからサンプリングを開始します。
    """
    _clk = clk        # ローカル変数にコピーしてアクセスを高速化
    _rx_data = rx_data
    
    # 最初のエッジ同期 (Sync with 1st bit) ---
    # まずCLKがHighであることを確認（すでにLowに落ちていた場合の誤読防止）
    # ※ もしREQからCLKまでが極端に速い場合はここを調整
    while _clk.value() == 0:
        pass
        
    # クロックの「立ち下がり（Falling Edge）」を待機
    while _clk.value() == 1:
        pass
    
    # 立ち下がった瞬間に1ビット目をサンプリング（D1の1ビット目）
    bits_buffer[0] = _rx_data.value()

    # 残り51ビットを回収 (51 bits loop) ---
    for i in range(1, 52):
        # クロックがHighに戻るのを待つ (Wait for High)
        while _clk.value() == 0:
            pass
        # クロックがLowに落ちるのを待つ (Wait for Falling Edge)
        while _clk.value() == 1:
            pass
        # 落ちた瞬間にサンプリング
        bits_buffer[i] = _rx_data.value()

    # 後処理 ---
    # 通信が終わってからREQ信号を解除（LowからHighへ）
    stop_request() 
    
    # ホスト（Rust側）へデータを送信
    send_to_host(bits_buffer)
    
    # 正常終了としてバリデート状態へ遷移
    return STATE_VALIDATE, ERR_NONE


@micropython.native
def process_receive_busy(bits_buffer):
    # メソッドをローカル変数に格納（これで辞書検索をスキップし、C言語並みに速くなる）
    _get_clk = clk.value
    _get_dat = rx_data.value
    
    # --- 同期開始 ---
    while _get_clk() == 0: continue
    while _get_clk() == 1: continue
    
    # 1ビット目
    bits_buffer[0] = _get_dat()

    # --- 残り51ビット ---
    for i in range(1, 52):
        while _get_clk() == 0: continue # High待ち
        while _get_clk() == 1: continue # Low待ち（エッジ検知）
        bits_buffer[i] = _get_dat()      # 【最速サンプリング】
    
    stop_request()
    send_to_host(bits_buffer)
    return STATE_VALIDATE, ERR_NONE


def process_validate(rx_buffer):
    """
    StateMachine の作法に合わせるバリデーション処理 (ラッパー)
    """
    validated = validator(rx_buffer)
    
    if validated:
        # 成功：ホストへ送って IDLE へ
        send_to_host(validated)
        return STATE_IDLE, ERR_NONE
    else:
        # 失敗：エラー状態へ
        return STATE_ERROR, ERR_DECODE


# デバッグ用に時間計測もりもりにしたやつ
# 本番ではこれは使わない (まあそもそもdebug用 branchだけど
def process_receive_t(bits_buffer):
    print("#DEBUG: start receive logic with timing")
    _clk = clk
    _data = rx_data
    TIMEOUT_US = 800  # 安全マージン大きめ

    # Request 出力タイムスタンプ
    t_req = time.ticks_us()
    send_request()  # 受信直前に再確認用
    print(f"#DEBUG: request sent at {t_req} us")

    # 最初の Clock 立下りを待つ
    while _clk.value() == 1:
        if time.ticks_diff(time.ticks_us(), t_req) > TIMEOUT_US * 100:
            print("#DEBUG: timeout waiting first falling edge")
            return STATE_ERROR, ERR_TIMEOUT

    t_first_falling = time.ticks_us()
    print(f"#DEBUG: first falling edge detected at {t_first_falling} us, T1={time.ticks_diff(t_first_falling, t_req)} us")

    # 最初のビット読み取り
    bits_buffer[0] = _data.value()
    led(LED_ON, LED_OFF, LED_OFF)

    for bit_count in range(1, BIN_FRAME_LENGTH):
        t_bit_start = time.ticks_us()

        # CLOCK High 待ち
        while _clk.value() == 0:
            if time.ticks_diff(time.ticks_us(), t_bit_start) > TIMEOUT_US:
                print(f"#DEBUG: bit {bit_count} timeout waiting High")
                return STATE_ERROR, ERR_TIMEOUT

        t_high = time.ticks_us()

        # CLOCK Low 待ち
        while _clk.value() == 1:
            if time.ticks_diff(time.ticks_us(), t_bit_start) > TIMEOUT_US:
                print(f"#DEBUG: bit {bit_count} timeout waiting Low")
                return STATE_ERROR, ERR_TIMEOUT

        t_falling = time.ticks_us()
        bits_buffer[bit_count] = _data.value()

        print(f"#DEBUG: bit {bit_count} read={bits_buffer[bit_count]}, High->Low={time.ticks_diff(t_falling, t_high)} us")

    stop_request()
    time.sleep_ms(50)
    led(LED_OFF, LED_OFF, LED_OFF)
    
    return STATE_VALIDATE, ERR_NONE

