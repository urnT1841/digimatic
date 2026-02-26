# 受け取ったバイナリフレーム 52bitをデジマチックフレームにデコードする
# nibble は lsbで受け取っているので反転したうえで処理

                
def to_int(nib):
    #msb想定
    return (nib[0] << 3) | (nib[1] << 2) | (nib[2] << 1) | nib[3]


def validate(nib13_frame):
    for i, nib in enumerate(nib13_frame):
        val = to_int(nib)
        
        # 個別ルール
        if i in CHECK_RULES:
            if not CHECK_RULES[i](val):
                raise ValueError(f"Position d{i+1} rule failed: expected {CHECK_RULES[i]}, got {val}")
        
        # BCD領域(d6~d11)の共通チェック
        elif 5 <= i <= 10:
            if not (0 <= val <= 9):
                raise ValueError(f"BCD out of range at d{i+1}: {val}")
    
    return True


def validator(bit_list):
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
    validated_frame = []

    #長さチェック    
    if len(bit_list) != REQUIRED_BIT_LENGTH:
        raise ValueError(f"abnormal frame length {len(bit_list)}")
    
    # 妥当な長さであったので 13個のnibble にスライス
    # ここでlsb -> msb に直しておく。詰めるのはタプル(中身をいじらないという意思)
    nib13_frame = [tuple(bit_list[i*4 : i*4+4][::-1]) for i in range(13)]

    # ここから本番バリデーションして，パスすればそのlistをBCD変換
    if validate(nib13_frame):
        # 通ったのでnibble -> bcd
        return validated_frame        



def decode_bin_frame(rx_frame):
    try:
        # validator 内部で raise されるので、
        # 正常系(ハッピーパス)の処理に集中
        decoded_values = validator(rx_frame)
        
        return decoded_values
        
    except ValueError as e:
        # どのバリデーションに引っかかったか、e にメッセージが入っている
        print(f"Validation Error: {e}")
    except Exception as e:
        # 予期せぬエラー（インデックスエラー等）
        print(f"Unexpected Error: {e}")    


    
    

def reverse_nibble(nib_list):
    """ lsb -> msb, 或いは msb -> lsb への並び替え
        arg:    list ["0","1","0","1"]
        return: list ["1","0","1","0"]   (arg list reversed)
        order: lsb -> sort to lsb 
               msb -> sort to msb (default)
    """
    # 受け取るのは正しい(問題のない) nibble前提
    return nib_list[::-1]



