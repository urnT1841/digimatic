import time
from micropython import const

from pin_definitions import rx_data, clk, req, req_sw, send_request, stop_request, ON, OFF
from led_switch import led_switch as led
from led_switch import LED_ON, LED_OFF
from decoder import BIN_FRAME_LENGTH, validator
from communicator import send_to_host,send_request, stop_request, get_command_from_pc, phy_sw_request

# StateMachine 状態定義
STATE_IDLE = const(0)
STATE_REQUEST = const(1)
STATE_RECEIVE = const(2)
STATE_VALIDATE = const(3)
STATE_ERROR = const(4)

#エラー定義
ERR_NONE = const(0)
ERR_TIMEOUT = const(1)  # 信号が来ない
ERR_READ    = const(2)  # クロックが途中で途切れた、物理的ノイズ  # TODO: 返す部分は未実装
ERR_DECODE  = const(3)  # バリデーション（FFFFヘッダ等）失敗


# 受信用バッファ
# 当初は 普通のリストを使っていたが,bit受信なので bytearray に変更
# ついでに52bitはきりが悪いので Byteの倍数として64Bit確保  → 12Bit足す
rx_buffer = bytearray(BIN_FRAME_LENGTH+12)

def process_idle():
    """  待ち受け    """    
    # 外部からの入力監視を行う
    next_state = STATE_IDLE

    cmd = get_command_from_pc()
    if cmd == "STOP":
        # STOP を受信したときはスクリプトを止める
        # main のFinnalyを実施後に停止となる
        raise SystemExit
    elif cmd == "REQ":
        next_state = STATE_REQUEST

    # 外部スイッチからのReq信号生成受付
    if phy_sw_request():
        next_state = STATE_REQUEST

    return next_state , ERR_NONE


def process_request():
    """ スイッチ, PCからのトリガを受け caliperにRequestを送る  """
    send_request()

    return STATE_RECEIVE , ERR_NONE


@micropython.native
def process_receive_busy(bits_buffer=rx_buffer):
    """
        取りこぼし対策として .native を使う

        とりあえず受信できているが,GPIOのレジスタ直読み等まだ改善は可能なので
        それも視野に入れておく
    """

    # メソッドをローカル変数に格納（これで辞書検索をスキップ。速くなる）
    _get_clk = clk.value
    _get_dat = rx_data.value
    
    # 受信
    for i in range(0, BIN_FRAME_LENGTH):
        while _get_clk() == 0:
            pass # High待ち
        while _get_clk() == 1:
            pass # Low待ち（エッジ検知）
        bits_buffer[i] = _get_dat()

    # ここで止めるのはホントは遅すぎるし処理重い
    stop_request()
    return STATE_VALIDATE, ERR_NONE


def process_validate(bits_buffer=rx_buffer):
    """
    StateMachine の作法に合わせるバリデーション処理 (ラッパー)
    """
    # 返値は成功時 デコードされた文字列 -> FFFF0~
    #      失敗時は None この時はErr Statusへ
    validated = validator(bits_buffer)
    
    if validated is not None:
        # 成功：ホストへ送って IDLE へ
        send_to_host(validated)
        return STATE_IDLE, ERR_NONE
    else:
        # 失敗：エラー状態へ
        return STATE_ERROR, ERR_DECODE



def process_err_handler():
    #未実装 エラーハンドリングを行う
    # 下記は体裁を整えただけ
    return STATE_IDLE, ERR_NONE

# 状態定義
state_map = {
    STATE_IDLE: process_idle,
    STATE_REQUEST: process_request,
    STATE_RECEIVE: process_receive_busy, # 引数がある場合はlambdaで包む
    STATE_VALIDATE: process_validate,
    STATE_ERROR: process_err_handler,         # 未実装
}
