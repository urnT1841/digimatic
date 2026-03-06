# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

from validation_rule import CHECK_RULES
from validation_rule import STATE_IDLE, STATE_ERROR, STATE_RECEIVE, STATE_REQUEST, STATE_VALIDATE
from validation_rule import ERR_DECODE, ERR_NONE, ERR_READ, ERR_TIMEOUT

BIN_FRAME_LENGTH = 52   # デジマチックのフレームは 52bit


def validate(nib13_frame):
    print("DEBUG: start validate")
    for i, nib in enumerate(nib13_frame):
        val = to_int(nib)
        
        # 個別ルール
        if i in CHECK_RULES:
            if not CHECK_RULES[i](val):
                return None

        # BCD領域(d6~d11)の共通チェック
        elif 5 <= i <= 10:
            if not (0 <= val <= 9):
                return None

    return True

#
# pico では使わない。もっと強力なマイコンならいけるか？
# いやNGって決まったわけじゃないけど
def validator_list_Comprehension(bit_list):
    """
    受信した52ビットの検証
    digimaticの意味付けに戻す
      長さ：13 (52Bit = 4bit x 13)
      d1~d4(header) : ALL 1 -> "F"
      d5:(sign) : BCD数値( 0 or 8 only)
      d6~d11(data) : BCD数値(0~9)
      d12(pointPos) : BCD数値(0~9)
      d13(unit) : BCD数値(0~9)
    """

    REQUIRED_BIT_LENGTH = 52

    #長さチェック    
    if len(bit_list) != REQUIRED_BIT_LENGTH:
        return STATE_ERROR, ERR_DECODE
    
    # 妥当な長さであったので 13個のnibble にスライス
    # ここでlsb -> msb に直しておく。詰めるのはタプル(中身をいじらないという意思)
    # リスト内包記法を使っているのでもしかしたら重いかもということでこれはやめる
    nib13_frame = [tuple(bit_list[i*4 : i*4+4][::-1]) for i in range(13)]

    # ここから本番バリデーションして，BCD変換
    # 実際のチェックは validate で実施
    if validate(nib13_frame):
        # 通ったのでnibble -> bcd, かつ13Byteの連続した文字列
        print("DEBUG: complete validate. make digi-strings")
        validated = to_bcd_output(nib13_frame)
        return validated
    else:
        # バリテーション失敗
        print("DEBUG: false validae")
        return None


def validator(bit_list):
    """
    受信したbit列の検証とBCD変換
    リスト内包記法による重い スライスやリスト作成などの処理を行わないようにした版   
    """

    REQUIRED_BIT_LENGTH = 52
    
    # 長さチェック（raiseを排除）
    if len(bit_list) != REQUIRED_BIT_LENGTH:
        return STATE_ERROR, ERR_DECODE
    
    results = []
    
    for i in range(13):
        base = i * 4
        # あの「一行」のビット演算
        val = (bit_list[base + 3] << 3) | (bit_list[base + 2] << 2) | \
              (bit_list[base + 1] << 1) | bit_list[base]
        
        # 個別ルールチェック
        if i in CHECK_RULES:
            if not CHECK_RULES[i](val):
                return None

        # BCDデータ領域チェック
        elif 5 <= i <= 10:
            if not (0 <= val <= 9):
                return None
        
        results.append("F" if val == 15 else str(val))
    
    return "".join(results)




def to_bcd_output(nib13_list):
    # all 1(15)はF, 他は intへの変換でOK
    digi_frame_str = ["F" if to_int(nib) == 15 else str(to_int(nib)) for nib in nib13_list]

    return "".join( digi_frame_str)


# 以下の関数はリスト内法表記にしたので使わなくなった
# が，まだデバッグで使うかもなので残しておく
def reverse_nibble(nib_list):
    """ lsb -> msb, 或いは msb -> lsb への並び替え
        arg:    list ["0","1","0","1"]
        return: list ["1","0","1","0"]   (arg list reversed)
        order: lsb -> sort to lsb 
               msb -> sort to msb (default)
    """
    # 受け取るのは正しい(問題のない) nibble前提
    return nib_list[::-1]


def to_int(nib):
    #msb想定
    return (nib[0] << 3) | (nib[1] << 2) | (nib[2] << 1) | nib[3]


