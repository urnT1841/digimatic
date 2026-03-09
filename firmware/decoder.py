# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

from validation_rule import CHECK_RULES
from state_process import ERR_DECODE, STATE_ERROR

def validator(bits_buffer):
    """
    受信したbit列の検証とBCD変換
    リスト内包記法による重いスライスやリスト作成などの処理を行わないようにした版
    52bitはよろしくないので64bitで処理。ただし使うのは52Bit迄
    """

    REQUIRED_BIT_LENGTH = 52  # ビット列の長さ これを超えるものは無視する
    NIBBLE = 4
    NIBBLE_CHUNK_COUNT = REQUIRED_BIT_LENGTH // NIBBLE   # デジマチックフレーム の nibble の数 4bit x 13 = 52

    # 長さチェック 長さの合わないものははじく
    # 元はチェックしていたが今は頭のビットから52bitまでの処理とする

    try:
        results = []
    
        for i in range(NIBBLE_CHUNK_COUNT):
            base = i * NIBBLE
            val = (bits_buffer[base + 3] << 3) | (bits_buffer[base + 2] << 2) | \
                  (bits_buffer[base + 1] << 1) | bits_buffer[base]
        
            # 個別ルールチェック
            if i in CHECK_RULES:
                if not CHECK_RULES[i](val):
                    return STATE_ERROR, ERR_DECODE

            # BCDデータ領域チェック
            # マジックナンバーの意味は validation_ruleみること
            elif 5 <= i <= 10:
                if not (0 <= val <= 9):
                    return STATE_ERROR, ERR_DECODE
        
            # ALL 1 は F とする
            results.append("F" if val == 15 else str(val))
    
        return "".join(results)
    
    except IndexError:
        # 52bitに満たないバッファが渡された時のセーフティネット
        return STATE_ERROR, ERR_DECODE
