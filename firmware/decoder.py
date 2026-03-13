
import struct


# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

# ルールは「人間が読める MSB 形式 → デバック楽
IS_BCD = lambda v: 0 <= v <= 9
MSB_CHECK_RULES = {
    0: lambda v: v == 0xF, 1: lambda v: v == 0xF, 
    2: lambda v: v == 0xF, 3: lambda v: v == 0xF,
    4: lambda v: v in (0, 8), # Sign (MSB: 0 or 8)
    5: IS_BCD, 6: IS_BCD, 7: IS_BCD, 8: IS_BCD, 9: IS_BCD, 10: IS_BCD,
    11: lambda v: 0 <= v <= 5, # PointPos
    12: lambda v: v in (0, 1),  # Unit
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


def decode_frame(nibbles, mode="MSB"):
    """
    検証済みニブル列を送信形式に変換
    mode="LSB": 物理ニブル列をそのまま返す
    mode="MSB": 人間が読める文字列 (ex: "FFFF064721020") に変換して返す
    """
    if mode == "MSB":
        # 文字列に変換 (FFFF0...)
        return "".join(["F" if v == 15 else hex(v)[2:].upper() if v > 9 else str(v) for v in nibbles])
    else:
        # LSB(BIN)モード: 13個のニブル(0-15)を8バイト(64bit)にパッキング
        
        # 13個のニブルをそのまま 1byte ずつ計13byteで送るか、
        # あるいは 4bit ずつ詰めるか。ここではシンプルに 13byte バイナリにします
        return struct.pack('13B', *nibbles)
