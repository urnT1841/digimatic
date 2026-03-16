import sys
import select

from pin_definitions import req_sw, data_btn


def get_command_from_pc():
    """ serialを監視  """

    # PCから REQ, STOP などの 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        return sys.stdin.readline().strip()
    return None


# 外部ボタンのリストと初期状態設定 (PullUpされているのでON(1) がデフォ )
# 1:押されていない , 0: 押下
buttons = [req_sw, data_btn]
last_sw_states = [1] * len(buttons)

def phy_sw_request():
    """
    Databボタンやタクトスイッチなど外部からReqを出せるように
    pull-upているPinに対してスイッチが押されることでGNDへ落ちる
      → send_request を送る (mainで State_request になってprocess_requestへ入る)
    """
    global last_sw_states
    sw_pressed = False

    for i in range(len(buttons)):
        current_val = buttons[i].value()
        # 立ち下がりエッジ (1 -> 0) を検知
        if last_sw_states[i] == 1 and current_val == 0:
            sw_pressed = True
        
        last_sw_states[i] = current_val

    # チャタリング対策は不要
    #  Trueの場合(押下検知) は Stateが変更されるのでちゃたってもここのロジックを通らない
    #  Falseの場合 Idle に入って100ms戻ってこないのでとうにチャタリングは収まっている
    return sw_pressed


def send_to_host(data):
    """
    data が文字列なら print (USB-CDC経由)
    data がバイナリなら sys.stdout.buffer.write (バイナリ直送)
    """
    if isinstance(data, str):
        print(data) # 改行込みで送信
    else:
        sys.stdout.buffer.write(data) # バイナリをそのまま流す
        sys.stdout.buffer.flush()
