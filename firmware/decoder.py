# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

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
                    return None

            # BCDデータ領域チェック
            # マジックナンバーの意味は validation_ruleみること
            elif 5 <= i <= 10:
                if not (0 <= val <= 9):
                    return None
        
            # ALL 1 は F とする
            results.append("F" if val == 15 else str(val))
    
        return "".join(results)
    
    except IndexError:
        # 52bitに満たないバッファが渡された時のセーフティネット
        return None
