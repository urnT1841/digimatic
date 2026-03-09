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
