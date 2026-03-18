# これは何?
これはミツトヨのデジタルキャリパーからの出力をPCで受けられるようにするツールです。
ノギスからの信号は RaspberryPi Pico (実際は seeed RP2040) が受けて PCに送ります。
PC側は Rust ，ノギス→PCへの間に raspberryPi Pico (実際にはXiao RP2040) です。

ノギス
  →
レベルシフタ(SN74LXC8T245PWR) をDip化
  →
RP Pico
   → 
PC (Linux / Windows)


## 現状
今の段階：
  - Simとして動きます (socatを使うのでLinux用)
      + ノギスの測定データ(ぽいものを生成)
      + ミツトヨ仕様に準じたフレーム文字列を生成
      + 送受信用にそれぞれ仮想ポートを生成
      + ポートを介してデータを送受信
      + フレームをデコードして測定値を得る
  
  - ノギス実機との接続
      + breadBord上では通信を確認 → ユニバーサル基板へ組み込み中

  ‐ software
      + Pico (micropython) : ノギスbit列受信 → PCへ送出
      + PC (Rust) : 文字列フレーム受信 → csv記録 / ターミナル表示
                    表示用ウィンドウへの表示 実装中


## Software
  - PC
  - rp pico :firmware 
              {main.py, pin_definitions.py, led_switch.py, state_process.py, 
               decoder.py, communicator.py}
            をPicoへ転送してください。picoを再起動するとmainが起動します。




## ノギスとの接続に使うデバイス
実機(デジタルノギス)との接続環境構築(電子工作)
  - ノギスとの接続ケーブル:ミツトヨ純正平形ストレート 905338
  - ケーブルoutput受け:ボックスヘッダ 10P 基板取付け (マルツオンライン 217010SE)
  - データ受けマイコン:seeed XIAO-RP2040
  - キャリパーからの信号レベルシフタ(1.5V -> 3.3V):SN74LXC8T245PWR
  - TSSOP24 → DIP変換基板:DA-TSSOP24-P65
  - レベルシフタへの電源供給：LDO AP2112

## TODO
[ ] シリアル通信の統合: generator を CdcReceiver に差し替え、実機の数値を DisplayApp に流す。
[ ] 設定UIの実装: フォントサイズや色の変更を egui のウィンドウから行えるようにする。
[ ] 単位表示の追加: 数値の横に "mm" などの単位を添える