# Legacy移行メモ（Digimaticプロジェクト）

本ドキュメントは、Sim / 仮想ポート系コードの整理および legacy 移行判断に関する設計判断ログである。

---

## 1. 背景

当初システムは以下構成であった：

- generator → frame builder → sender → receiver → display
- 物理層：serialport + socat 仮想ポート
- 目的：デジマチック風フレームの完全再現

しかし開発途中で以下に移行：

- mpscベース通信（Rust内部完結）
- GUI連携優先設計

結果として一部モジュールが未使用化した。

---

## 2. legacy化対象モジュール

### sim/sender.rs

#### 内容
- SerialPortへ直接送信するAPI
- SendMode（SimpleText / DigimaticFrame）

#### 状態
- 現在未使用
- mpsc化により経路が完全に置換

#### 判断
- legacy行き
- 将来：仮想ポート復活時に再利用可能

---

### sim/port_prepare.rs

#### 内容
- socatによる仮想TTY生成
- tx/rxペア作成

#### 状態
- Linux依存
- 現在完全未使用

#### 判断
- legacy行き
- OS依存層として将来再設計候補

---

## 3. simディレクトリ方針

### 現状
sim = Simulationの短縮として運用

### 懸念
- 意味が弱い（simulationの方が明確）
- ただし既存コード依存が多い

### 判断
- 変更しない（現時点維持）
- 将来リネーム検討

---

## 4. frame builderの位置付け

### 現状
- f64 → Digimaticフレーム配列生成

### 判断
- 今は配列ベースで維持
- 将来バイナリフレーム追加時にtrait化検討
- そのタイミングで再設計

---

## 5. pub / private 設計の方針

### 方針整理

- parser / frame / measurement
  → 外部公開（crate内部APIとして安定）

- sim系
  → 基本非公開 or legacyへ移行

- logger / config / presentation
  → アプリ層（UI/IO）として公開

- internal helper
  → crate内限定（pub(crate)）

---

## 6. console出力の扱い

### 論点
- error / warn → errors.rs
- info → presentation or logger
- console.rs単体切り出しは未実施

### 結論
- 現時点では分離しない
- 将来 UI/ログ統合時に再編

---

## 7. bin / legacy の扱い整理

### src/bin
- Rust標準の「別バイナリ」
- 特別機能ではない（ただのcargo規約）

### legacyディレクトリ
- Rustの標準機能ではない
- プロジェクト内ルール
- 「現行未使用だが保持したいコード置き場」

---

## 8. 最終判断

- pub/private整理：完了
- sim/port/sender：legacy移行
- frame builder：現状維持
- config / errors：V3で再整理予定
- 全体構造：現状でPR可能状態

---

## 9. 今後の拡張ポイント

- binary frame generator trait化
- OS依存IO層の再設計（virtual port復活）
- logger / console統合設計
- GUI/CLI共通presentation層整理