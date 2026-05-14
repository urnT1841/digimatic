# ToDo list

## 🟦 PHASE 1: V2 リリース（現在進行中：コードフリーズ優先）
- [ ] **APIリネームと統合**
    - [✅] `MeasurementRead` トレイトのメソッドを `read_measurement` に変更
    - [✅] `CdcReceiver` 内部のバイト読み取りを `read_raw_frame` にリネーム
    - [ ] 戻り値を `String` から `Measurement` 構造体へ変更
- [ ] **ロジックの隠蔽（カプセル化）**
    - [ ] `CdcReceiver::read_measurement` 内で Str/Bin の判定を完結させる
    - [ ] `data_receiver`（呼び出し側）の `match` 分岐を撤去し、シンプルにする
- [ ] **Sim（シミュレータ）の整理**
    - [ ] Simのスレッドとチャンネルを撤去
    - [ ] 呼び出されたらその場でデータを生成して返す同期的な `SimReceiver` へ変更
- [ ] **表示・出力のクリーンアップ**
    - [ ] GUIモード時のターミナル出力（println!）を抑制する
    - [ ] 必要な時だけデバッグ表示を復活できる仕組み（コメントアウトやフラグ

## 🟨 PHASE 2: V3（アーキテクチャの進化）
- [ ] **Event-driven（イベント駆動）への移行**
    - [ ] `DigimaticEvent`（DataReceived, Error, etc.）の定義
    - [ ] パイプラインを「購読型（Pub/Sub）」に再設計
- [ ] **表示・出力の柔軟な切り替え**
    - [ ] ターミナル表示、GUI表示、ログ保存を「イベントリスナー」として独立させる
    - [ ] 実行中に表示のON/OFFを切り替えられる機能
- [ ] **GUI拡張**
    - [ ] イベント駆動に合わせたGUI側のデータ受信最適化