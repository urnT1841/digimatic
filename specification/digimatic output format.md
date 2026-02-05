# digimatic output format


## Pins

| PinNo. | Signal | I/O | description |
|--------| ------ | --- | ----------- |
| 1      | GND    |  -  | Signal GND  |
| 2      | DATA   |  O  | mes data    |
| 3      | CK     |  O  | Clock out   |
| 4      | N.C.   |  -  | Non connect |
| 5      | REQ    |  I  | deta Request|
| 6      | ORIG   |  I  | origin sig. |
| 7      | N.C    |  -  | Non connect |
| 8      | N.C    |  -  | Non Connect |
| 9      | +5V    |  -  | power sup.  |
| 10     | GND(F.G) | - | Flame GND   |


## electrical spec
### output terminal : CK, DATA
    - Nch open drain
    - Max output current: 400uA (@VOL=0.4V)
    - output BV : -0.3 ~ 7V

### input terminal : REQ, ORIG
    - COMS input with pull up resi.
    - inner Voltage: Vdd = 1.35 ~ 1.65V
    - Pull up resi. : R1 = 10 ~ 100KΩ
    - "H" level input: Vh = 1.1V min.
    - "L" level input: Vl = 0.3V max.


### Data Format
    - 4bit = 1digits(d)  d1 ~ d13 までの 13デジット
    - 各デジットは最下位ビット(LSB) から 最上位ビット(MSB) の順に出力します。

    d1: All F (1111)
    d2: All F (1111)
    d3: All F (1111)
    d4: All F (1111)
    d5: sign +:0(0000) , -:8(1000)
    d6: mes data (xxxx)
    d7: mes data (xxxx)
    d8: mes data (xxxx)
    d9  mes dat (xxxx)
    d10: mes data (xxxx)
    d11: mes data (xxxx)
    d12: 小数点位置(1~5) (※1)
    d13 unit 0:mm, 1:inch

    ※1例：0 -> 000000.
           1 -> 00000.0
           5 -> 0.00000

### timing chart

    | symbol | min. | Max. | Description                    |
    | ------ | ---- | ---- | ------------------------------ |
    | t1     | 0    | 2    | request -> data output star    |
    | t2     | 15u  | -    | ck [L] level pulse width       |
    | t3     | 100u | -    | ck [H] level pulse width       |
    | t4     | 100u | -    | ck down to est.data pulse width|
    | t5     | 0    | -    | reqest [L] min hold time       | 
    
    | t6
    | t8
    | t8
    | t9