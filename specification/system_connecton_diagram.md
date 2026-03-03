graph LR
    %% ノギス
    subgraph "ノギス CD-15AXW"
        DATA
        CLOCK
        REQ
        GND1[GND]
    end

    %% LDO
    subgraph "AP2112K<br>(LDO)"
        VCC1["デバイス<br>AP2112K"]
    end

    %% レベルシフタ
    subgraph "SN74LXC8T245<br>LVShifter"
        direction LR
        DATA_IN("DATA IN") --> DATA_OUT("DATA OUT")
        CLOCK_IN("CLOCK IN") --> CLOCK_OUT("CLOCK OUT")
    end

    %% Pico
    subgraph "Pico"
        GPIO_DATA["GPIO DATA"]
        GPIO_CLOCK["GPIO CLOCK"]
        GPIO_REQ["GPIO REQ"]
        GND2[GND]
    end

    %% 配線（信号線とプルアップ）
    DATA --> DATA_IN
    DATA_OUT --> GPIO_DATA
    CLOCK --> CLOCK_IN
    CLOCK_OUT --> GPIO_CLOCK
    REQ --> GPIO_REQ

    %% プルアップ線（青の点線）
    DATA -.-> VCC1
    CLOCK -.-> VCC1
    REQ -.-> VCC1

    %% REQシリーズ抵抗は赤線で
    REQ -->|1.5kΩ| GPIO_REQ

    %% GND共通
    GND1 --- GND2

    %% 色付けクラス定義（ノード色）
    classDef high fill:#d4f0d4,stroke:#2d8f2d,color:#000
    classDef low fill:#f0d4d4,stroke:#8f2d2d,color:#000
    classDef hiz fill:#f0f0d4,stroke:#b5a200,color:#000

    %% ノード色付け
    class REQ hiz
    class GPIO_REQ low
    class DATA_OUT hiz
    class CLOCK_OUT hiz