import time
import gc
from micropython import const

import pin_definitions as pdef
from decoder import BIN_FRAME_LENGTH, validator, decode_frame
from communicator import send_to_host, get_command_from_pc, phy_sw_request
import diag as diag
from i18n import t


# StateMachine 状態定義
STATE_IDLE = const(0)
STATE_REQUEST = const(1)
STATE_RECEIVE = const(2)
STATE_VALIDATE = const(3)
STATE_ERROR = const(4)
STATE_SWITCH = const(5)
STATE_DIAG = const(6)
STATE_SIM = const(7)
STATE_STOP = const(8)
STATE_CONFIG = const(9)

#エラー定義
ERR_NONE = const(0)
ERR_TIMEOUT = const(1)  # 信号が来ない
ERR_READ    = const(2)  # クロックが途中で途切れた、物理的ノイズ  # TODO: 返す部分は未実装
ERR_DECODE  = const(3)  # バリデーション（FFFFヘッダ等）失敗

class DigimaticSession:
    # 以下の2つの組み合わせのみを想定
    MODE_STR = ("MSB", "STR")
    MODE_BIN = ("LSB", "BIN")

    def __init__(self):
        # 設定 (Config)
        self.config = self.MODE_STR      # nibble_maker / validator 用

        self.rx_buffer = bytearray(BIN_FRAME_LENGTH + 12)   # 受信生ビット 52Bit+12Bit(Padding分) => 64Bit確保
        self.nibbles = []      # 検証済み数値リスト
    
    @property
    def mode(self):
        return self.config[0]   # MSB or LSB
    
    @property
    def format(self):
        return self.config[1]   # Str or Bin  PCに送るフォーマット

    def reset_data(self):
        """データ受信前にバッファをクリア"""
        self.rx_buffer[:] = b'\x00' * len(self.rx_buffer)
        self.nibbles = []

# ここで実体生成
session = DigimaticSession()

def mode_switcher(cmd):
    if cmd == "BIN":
        session.config = session.MODE_BIN
    elif cmd == "STR":
        session.config = session.MODE_STR


def process_idle():
    """ 
    Mapを使ってFlatな待ち受け
    """

    cmd = get_command_from_pc()    
    # 状態遷移check
    if cmd in CMD_TO_STATE_MAP:
        return CMD_TO_STATE_MAP[cmd], ERR_NONE
        
    # Action CHeck
    if cmd in CMD_ACTION_MAP:
        CMD_ACTION_MAP[cmd]()
        return STATE_IDLE, ERR_NONE

    # Physical sw check
    # 初めは100ms ぼーっと待っていたが物理スイッチチェックは粒度を細かく
    for _ in range(5):
        if phy_sw_request():
            time.sleep_ms(5)  # ノイズ除去 (debounce)
            if phy_sw_request():
                while phy_sw_request():  # 押しっぱなし対策  wait for btn release
                    time.sleep_ms(5)
                return STATE_REQUEST, ERR_NONE
        time.sleep_ms(20)

    return STATE_IDLE, ERR_NONE


def process_request():
    """ スイッチ, PCからのトリガを受け caliperにRequestを送る  """
    # sessionをリセットしてから受信開始
    session.reset_data()
    
    pdef.send_request()

    return STATE_RECEIVE , ERR_NONE


@micropython.native
def process_receive_busy():
    """
        取りこぼし対策として .native を使う

        とりあえず受信できているが,GPIOのレジスタ直読み等まだ改善は可能なので
        それも視野に入れておく
    """

    # メソッドをローカル変数に格納（これで辞書検索をスキップ。速くなる）
    _get_clk = pdef.PINS["clk"].value
    _get_dat = pdef.PINS["rx_data"].value
    _ticks_us = time.ticks_us
    _ticks_diff = time.ticks_diff
    _bits = session.rx_buffer
    
    TIMEOUT_US = 500_000  # 500ms
    
    # 受信
    for i in range(0, BIN_FRAME_LENGTH):
        t = _ticks_us()
        while _get_clk() == 0:  # High待ち
            if _ticks_diff(_ticks_us(), t) > TIMEOUT_US:
                pdef.stop_request()
                return STATE_ERROR, ERR_TIMEOUT
        while _get_clk() == 1:
            pass # Low待ち（エッジ検知）
        _bits[i] = _get_dat()

    # ここで止めるのはホントは遅すぎるし処理重い
    pdef.stop_request()
    return STATE_VALIDATE, ERR_NONE


def process_validate():
    """
    StateMachine の作法に合わせるバリデーション処理 (ラッパー)
    """
    # 返値は成功時 デコードされた文字列 -> FFFF0~
    #      失敗時は None この時はErr Statusへ
    validated = validator(session.rx_buffer, session.mode)
    
    if validated is not None:
        # 成功：ホストへ送って IDLE へ
        send_frame = decode_frame(validated, session.mode)
        send_to_host(send_frame)
        return STATE_IDLE, ERR_NONE
    else:
        # 失敗：エラー状態へ
        return STATE_ERROR, ERR_DECODE


def process_diag_handler():
    # dig mode へはいる
    print(f"\n{t('enter_diag')}")
    diag.main_loop(diag.MENU_OPTIONS)
    # dig mode から出てくる
    print(f"\n{t('finish_diag')}")
    
    #階層メニューで再起を使っているので。念のためGC実施
    gc.collect()

    from main import splash_welcome
    splash_welcome()

    return STATE_IDLE, ERR_NONE


def process_sim_handler():
    """ Sim モード SimでFrame迄組み立ててPCへ送る"""
    import model_caliper

    # 測定データフレーム生成，文字列化
    val_str = model_caliper.sim_measure()
    frame_list = model_caliper.build_frame(val_str)
    send_frame = "".join([str(v) for v in frame_list])

    send_to_host(send_frame)

    # SIM継続判定
    time.sleep_ms(500)
    cmd = get_command_from_pc() #
    if cmd == "STOP":
        return STATE_IDLE, ERR_NONE
        
    #念のためGC実施
    gc.collect()

    return STATE_SIM, ERR_NONE

def process_stop_handler():
    """ 終了処理ハンドラー """

    print(f"\n{t('stopped_sys')}")
    
    # 最終的に例外を投げてメインループを抜ける
    raise SystemExit


def process_config_handler():
    print(t("CONFIG"))
    # 実際の設定はconfig.py でやる
    from config import run_interactive_menu
    run_interactive_menu()

    return STATE_IDLE, ERR_NONE

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
    STATE_STOP: process_stop_handler,
    STATE_ERROR: process_err_handler,         # TODO:未実装
    STATE_DIAG: process_diag_handler,
    STATE_CONFIG: process_config_handler,
    STATE_SIM: process_sim_handler,
}

#コマンド受付マッピング
CMD_TO_STATE_MAP = {
    "STOP": STATE_STOP,  # 終了状態へ
    "REQ":  STATE_REQUEST,
    "DIAG": STATE_DIAG,
    "SIM":  STATE_SIM,
    "CONFIG": STATE_CONFIG,
}

CMD_ACTION_MAP = {
    "BIN": lambda: mode_switcher("BIN"),
    "STR": lambda: mode_switcher("STR"),
}