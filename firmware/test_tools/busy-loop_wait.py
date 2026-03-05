def receive_busy(bits_buffer):
    """
    Busy-loop でクロック同期受信するシンプル版
    - Requestは事前に送信済み、または最初に送る
    - Pythonの処理遅延を極力排除
    - bits_buffer: 長さ BIN_FRAME_LENGTH のリスト
    """
    _clk = clk
    _data = rx_data

    # Requestを出しておく
    send_request()

    # 最初のClock立下り待ち
    while _clk.value() == 1:
        pass

    # ビット受信ループ
    for i in range(BIN_FRAME_LENGTH):
        # CLOCK High 待ち
        while _clk.value() == 0:
            pass
        # CLOCK Low 待ち
        while _clk.value() == 1:
            pass
        # データ読み取り
        bits_buffer[i] = _data.value()

    # 受信完了後にLEDやデバッグ出力
    led(LED_ON, LED_OFF, LED_OFF)
    time.sleep_ms(50)
    led(LED_OFF, LED_OFF, LED_OFF)