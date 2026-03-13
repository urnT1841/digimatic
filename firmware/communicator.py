import sys
import select

from pin_definitions import req_sw


def get_command_from_pc():
    """ serialを監視  """

    # PCから REQ, STOP などの 文字列が送られてくることを期待
    if select.select([sys.stdin], [], [], 0)[0]:
        return sys.stdin.readline().strip()
    return None


last_sw_state = 1  # 1:押されていない , 0: 押下
def phy_sw_request():
    global last_sw_state
    """
      Databボタンやタクトスイッチなど外部からReqを出せるように
    """
    # pull-upているPinに対してスイッチが押されることでGNDへ落ちる
    # → send_request を送る (mainで State_request になってprocess_requestへ入る)

    current_sw_state = req_sw.value()

    # 離れていて押した最初だけ反応
    sw_pressed = last_sw_state == 1 and current_sw_state == 0
    last_sw_state = current_sw_state

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
