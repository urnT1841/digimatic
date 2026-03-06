
# StateMachine 状態定義
STATE_IDLE = 0
STATE_REQUEST = 1
STATE_RECEIVE = 2
STATE_VALIDATE = 3
STATE_ERROR = 4

#エラー定義
ERR_NONE = 0
ERR_TIMEOUT = 1  # 信号が来ない
ERR_READ    = 2  # クロックが途中で途切れた、物理的ノイズ  # TODO: 返す部分は未実装
ERR_DECODE  = 3  # バリデーション（FFFFヘッダ等）失敗



# バリデーションルール
# d6~d11 (index 5~10) はすべてBCD (0-9)
CHECK_RULES = {
    0: lambda v: v == 0xF, # d1: Header
    1: lambda v: v == 0xF, # d2
    2: lambda v: v == 0xF, # d3
    3: lambda v: v == 0xF, # d4
    4: lambda v: v in (0, 8), # d5: Sign
    # d6~d11 はループ内で一括定義もありだが
    # わかりやすさのために個別に、あるいは判定ロジック側で処理
    11: lambda v: 0 <= v <= 5, # d12: PointPos
    12: lambda v: v in (0, 1),  # d13: Unit
    }
