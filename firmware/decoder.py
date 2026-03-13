# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

# ルールは「人間が読める MSB 形式 → デバック楽
IS_BCD = lambda v: 0 <= v <= 9
CHECK_RULES = {
    0: lambda v: v == 0xF, 1: lambda v: v == 0xF, 
    2: lambda v: v == 0xF, 3: lambda v: v == 0xF,
    4: lambda v: v in (0, 8), # Sign (MSB: 0 or 8)
    5: IS_BCD, 6: IS_BCD, 7: IS_BCD, 8: IS_BCD, 9: IS_BCD, 10: IS_BCD,
    11: lambda v: 0 <= v <= 5, # PointPos
    12: lambda v: v in (0, 1),  # Unit
}

def nibble_maker(bits, mode="LSB"):
    """
    bits: [b0, b1, b2, b3] (b0が最初、b3が最後)
    """
    NIBBLE = 4
    b = bits[:NIBBLE]

    if mode == "LSB":
        # 受信したビット順を「そのまま」数値のビット位置に反映させる
        # 例: [1, 0, 0, 0] が届いたら、数値の 2^0桁に1、他は0。
        # つまり、受信bit列 1000 は 1000 (数値としての8) として扱う
        # ※ここでの「そのまま」は、b[0]を最上位ビットにする動作
        return (b[0] << 3) | (b[1] << 2) | (b[2] << 1) | b[3]
    
    else: # MSBオプション
        # 物理順を反転させて、人間が読む数値（MSB重み）にする
        # 例: [1, 0, 0, 0] が届いたら、それを 1 (0001) に変換する
        return b[0] | (b[1] << 1) | (b[2] << 2) | (b[3] << 3)


# ルール定義
# MSB: 人間が仕様書で見る数値
MSB_CHECK_RULES = {
    0: lambda v: v == 0xF, 1: lambda v: v == 0xF, 2: lambda v: v == 0xF, 3: lambda v: v == 0xF,
    4: lambda v: v in (0, 8),      # Sign (+:0, -:8)
    5: lambda v: 0 <= v <= 9, 6: lambda v: 0 <= v <= 9, 7: lambda v: 0 <= v <= 9,
    8: lambda v: 0 <= v <= 9, 9: lambda v: 0 <= v <= 9, 10: lambda v: 0 <= v <= 9,
    11: lambda v: 0 <= v <= 5,     # Pos
    12: lambda v: v in (0, 1)       # Unit
}

# LSB: 物理ビット並びをそのまま数値化した時の数値
# 1(0001)->1, 2(0010)->2 ... ではなく、定義に従い 1000->8, 0100->4 となる世界
LSB_BCD_SET = (0, 8, 4, 12, 2, 10, 6, 14, 1, 9)
LSB_CHECK_RULES = {
    0: lambda v: v == 0xF, 1: lambda v: v == 0xF, 2: lambda v: v == 0xF, 3: lambda v: v == 0xF,
    4: lambda v: v in (0, 1),      # Sign (+:0, -:1) ※1000のLSB解釈は1
    5: lambda v: v in LSB_BCD_SET, 6: lambda v: v in LSB_BCD_SET, 7: lambda v: v in LSB_BCD_SET,
    8: lambda v: v in LSB_BCD_SET, 9: lambda v: v in LSB_BCD_SET, 10: lambda v: v in LSB_BCD_SET,
    11: lambda v: v in (0, 8, 4, 12, 2, 10), # Pos (0-5反転)
    12: lambda v: v in (0, 8)       # Unit (mm:0, inch:8)
}


def validator(bits_buffer, mode="LSB"):
    rules = LSB_CHECK_RULES if mode == "LSB" else MSB_CHECK_RULES
    nibbles = []
    try:
        for i in range(13):
            val = nibble_maker(bits_buffer[i*4 : (i+1)*4], mode)
            if i in rules and not rules[i](val):
                return None
            nibbles.append(val)
        return nibbles
    except Exception:
        return None



def decoder_and_sender(nibbles, mode="LSB"):
    """
    検証済みニブル列を送信形式に変換
    mode="LSB": 物理ニブル列(8bit×13)をそのままバイナリ等で送出(またはそのまま返す)
    mode="MSB": 人間が読める文字列 (ex: "FFFF064721020") に変換
    """
    if mode == "MSB":
        # MSB(STR)モード: ニブルを文字に変換
        # 0-9はその数字、15(0xF)は"F"、それ以外(符号や小数点等)も一旦16進数表記
        frame = []
        for v in nibbles:
            if 0 <= v <= 9:
                frame.append(str(v))
            elif v == 15:
                frame.append("F")
            else:
                # 符号(8)や小数点(2/4)などはそのまま16進文字へ
                frame.append(hex(v)[2:].upper())
        
        # 文字列として結合して送出
        output_str = "".join(frame)
        print(output_str) # または uart.write(output_str + "\n")
        return output_str

    else:
        # LSBモード: 物理ニブル(受信bit並びそのまま)をリストまたはバイナリで返す
        # Rust側へバイナリで送るための前段階
        # (例: struct.packなどでパッキングして送出)
        return nibbles


##
##  内部でデコードしないようにする版が完成するまで取っておく / 参照用
##
def old_validator(bits_buffer):
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
