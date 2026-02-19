# Seeed Studio XIAO RP2040 について

## product wiki
[Seeed Studio XIAO RP2040 入門ガイド](https://wiki.seeedstudio.com/ja/XIAO-RP2040/)

このファイルに書いた情報は基本的に上のリンクから引っ張ってきたデータ

### 特徴

- 強力なMCU：デュアルコアARM Cortex M0+プロセッサ、最大133MHzまでの柔軟なクロック動作
- 豊富なオンチップリソース：264KBのSRAMと2MBのオンボードフラッシュメモリ
- 柔軟な互換性：Micropython/Arduino/CircuitPythonをサポート
- 簡単なプロジェクト操作：ブレッドボード対応＆SMD設計、背面に部品なし
- 小型サイズ：ウェアラブルデバイスや小型プロジェクト向けの親指ほどの小ささ（21x17.8mm）
- 複数のインターフェース：11個のデジタルピン、4個のアナログピン、11個のPWMピン、1個のI2Cインターフェース、1個のUARTインターフェース、1個のSPIインターフェース、1個のSWDボンディングパッドインターフェース

### 仕様

| 製品名 |  Seeed Studio XIAO RP2040 |
|------- | ------------------------ |
|チップセット|Silicon - Raspberry Pi Documentation|
|プロセッサ|最大133MHzで動作するデュアルCortex M0+プロセッサ|
|RAM|264KB SRAM|
|Flash|2MB オンボードフラッシュ|
|interface|GPIOピン x14, デジタルピン x11,アナログピン x4,I2C x1, UART x1,SPI x1,PWM x11|
|on board|ユーザーLED（3色）x1, 電源LED x1, RGB LED x1, リセットボタン x1, ブートボタン x1|
|ワイヤレス| - |
|power|入力電圧（Type-C）：5V, 入力電圧（BAT）：5V|
|低消費電力モード| - |
|ソフトウェア互換性|Arduino、PlatformIO、MicroPython、CircuitPython、tinyGo、Rust、Zephyr、Exhibition for XIAO Series | Seeed Studio Wiki |
|動作温度|-20°C-70°C |
|寸法| 21x17.8mm|
|バリエーション| Seeed Studio XIAO RP2040 3PCS Pack | Save 10% for tiny Mic...
Seeed Studio XIAO RP2040 Pre-Soldered | Plug-and-Play mini ... |
|||

### ピンマップ

|XIAOピン|機能|チップピン|説明|
|--------|---|---------|----|
|5V      |VBUS| -|電源入力/出力|
|GND |||
|3V3 | 3V3_OUT| | 電源出力|
|D0 | Analog| P26  | GPIO、ADC|
|D1 | Analog| P27  | GPIO、ADC|
|D2 | Analog| P28  | GPIO、ADC|
|D3 | Analog| P29  | GPIO、ADC|
|D4 | SDA   | P6   | PIO、I2Cデータ|
|D5 | SCL   | P7   | GPIO、I2Cクロック|
|D6 | TX    | P0   | GPIO、UART送信|
|D7 | RX,CSn| P1   | GPIO、UART受信、CSn|
|D8 | SCK   | P2   | GPIO、SPIクロック|
|D9 | MISO  | P4   | GPIO、SPIデータ|
|D10 | MOSI | P3   | GPIO、SPIデータ|
|Reset | RUN	RUN||
|Boot | RP2040_BOOT|| ブートモード開始|
|CHARGE_LED| VCC_3V3|| CHG-LED_Red|
|RGB LED| NEOPIX ||	RGB LED|
|USER_LED_R|| IO17_RGB-R | ユーザー制御赤色RGB LEDピン|
|USER_LED_B|| IO25_RGB-B | ユーザー制御青色RGB LEDピン|
|USER_LED_G|| IO16_RGB-G | ユーザー制御緑色RGB LEDピン|
||||

<pre>
      [ USB-C ]
  D0 [1]     [14] 5V
  D1 [2]     [13] GND
  D2 [3]     [12] 3V3
  D3 [4]     [11] D10 (MOSI)
  D4 [5]     [10] D9  (MISO)
  D5 [6]     [09] D8  (SCK)
  D6 [7]     [08] D7  (CSn)
      [ R ] [ B ]
</pre>